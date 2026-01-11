//! Directory-based loading of known values from JSON registry files.
//!
//! This module provides functionality to load known values from JSON files
//! stored in configurable directories. It is only available when the
//! `directory-loading` feature is enabled (which is the default).
//!
//! # Overview
//!
//! The module supports loading known values from:
//! - A default directory: `~/.known-values/`
//! - Custom directories specified at runtime
//!
//! Values loaded from JSON files can override hardcoded values when they
//! share the same codepoint (numeric identifier).
//!
//! # JSON File Format
//!
//! Registry files should follow the BlockchainCommons format:
//!
//! ```json
//! {
//!   "ontology": {
//!     "name": "my_registry",
//!     "source_url": "https://example.com/registry"
//!   },
//!   "entries": [
//!     {
//!       "codepoint": 1000,
//!       "canonical_name": "myValue",
//!       "type": "property",
//!       "uri": "https://example.com/vocab#myValue",
//!       "description": "A custom known value"
//!     }
//!   ]
//! }
//! ```
//!
//! Only the `entries` array with `codepoint` and `canonical_name` fields
//! is required; other fields are optional.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Deserialize;

use crate::KnownValue;

/// A single entry in a known values JSON registry file.
#[derive(Debug, Deserialize)]
pub struct RegistryEntry {
    /// The unique numeric identifier for this known value.
    pub codepoint: u64,
    /// The canonical string name for this known value.
    pub canonical_name: String,
    /// The type of entry (e.g., "property", "class", "value").
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    /// An optional URI reference for this known value.
    pub uri: Option<String>,
    /// An optional human-readable description.
    pub description: Option<String>,
}

/// Metadata about the ontology or registry source.
#[derive(Debug, Deserialize)]
pub struct OntologyInfo {
    /// The name of this registry or ontology.
    pub name: Option<String>,
    /// The source URL for this registry.
    pub source_url: Option<String>,
    /// The starting codepoint for entries in this registry.
    pub start_code_point: Option<u64>,
    /// The processing strategy used to generate this registry.
    pub processing_strategy: Option<String>,
}

/// Root structure of a known values JSON registry file.
#[derive(Debug, Deserialize)]
pub struct RegistryFile {
    /// Metadata about this registry.
    pub ontology: Option<OntologyInfo>,
    /// Information about how this file was generated.
    pub generated: Option<GeneratedInfo>,
    /// The known value entries in this registry.
    pub entries: Vec<RegistryEntry>,
    /// Statistics about this registry (ignored during parsing).
    #[serde(default)]
    pub statistics: Option<serde_json::Value>,
}

/// Information about how a registry file was generated.
#[derive(Debug, Deserialize)]
pub struct GeneratedInfo {
    /// The tool used to generate this registry.
    pub tool: Option<String>,
}

/// Errors that can occur when loading known values from directories.
#[derive(Debug)]
pub enum LoadError {
    /// An I/O error occurred while reading files.
    Io(io::Error),
    /// A JSON parsing error occurred.
    Json {
        /// The file that caused the error.
        file: PathBuf,
        /// The underlying JSON error.
        error: serde_json::Error,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {}", e),
            LoadError::Json { file, error } => {
                write!(f, "JSON parse error in {}: {}", file.display(), error)
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io(e) => Some(e),
            LoadError::Json { error, .. } => Some(error),
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(error: io::Error) -> Self {
        LoadError::Io(error)
    }
}

/// Result of a directory loading operation.
#[derive(Debug, Default)]
pub struct LoadResult {
    /// Known values loaded, keyed by codepoint.
    pub values: HashMap<u64, KnownValue>,
    /// Files that were successfully processed.
    pub files_processed: Vec<PathBuf>,
    /// Non-fatal errors encountered during loading.
    pub errors: Vec<(PathBuf, LoadError)>,
}

impl LoadResult {
    /// Returns the number of unique values loaded.
    pub fn values_count(&self) -> usize {
        self.values.len()
    }

    /// Returns an iterator over the loaded known values.
    pub fn values_iter(&self) -> impl Iterator<Item = &KnownValue> {
        self.values.values()
    }

    /// Consumes the result and returns the loaded known values.
    pub fn into_values(self) -> impl Iterator<Item = KnownValue> {
        self.values.into_values()
    }

    /// Returns true if any errors occurred during loading.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Configuration for loading known values from directories.
///
/// This struct specifies which directories to search for JSON registry files.
/// Directories are processed in order, with values from later directories
/// overriding values from earlier directories when codepoints collide.
///
/// # Examples
///
/// ```rust,ignore
/// use known_values::DirectoryConfig;
///
/// // Use only the default directory (~/.known-values/)
/// let config = DirectoryConfig::default();
///
/// // Use custom paths
/// let config = DirectoryConfig::with_paths(vec![
///     "/etc/known-values".into(),
///     "/usr/share/known-values".into(),
/// ]);
///
/// // Use custom paths with default appended
/// let config = DirectoryConfig::with_paths_and_default(vec![
///     "/etc/known-values".into(),
/// ]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DirectoryConfig {
    /// Search paths in priority order (later paths override earlier).
    paths: Vec<PathBuf>,
}

impl DirectoryConfig {
    /// Creates a new empty configuration with no search paths.
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }

    /// Creates configuration with only the default directory (`~/.known-values/`).
    pub fn default_only() -> Self {
        Self {
            paths: vec![Self::default_directory()],
        }
    }

    /// Creates configuration with custom paths (processed in order).
    ///
    /// Later paths in the list take precedence over earlier paths when
    /// values have the same codepoint.
    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }

    /// Creates configuration with custom paths followed by the default directory.
    ///
    /// The default directory (`~/.known-values/`) is appended to the list,
    /// so its values will override values from the custom paths.
    pub fn with_paths_and_default(mut paths: Vec<PathBuf>) -> Self {
        paths.push(Self::default_directory());
        Self { paths }
    }

    /// Returns the default directory: `~/.known-values/`
    ///
    /// Falls back to `./.known-values/` if the home directory cannot be determined.
    pub fn default_directory() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".known-values")
    }

    /// Returns the configured search paths.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Adds a path to the configuration.
    ///
    /// The new path will be processed after existing paths, so its values
    /// will override values from earlier paths.
    pub fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }
}

/// Loads all JSON registry files from a single directory.
///
/// This function scans the specified directory for files with a `.json`
/// extension and attempts to parse them as known value registries.
///
/// # Arguments
///
/// * `path` - The directory to scan for JSON registry files.
///
/// # Returns
///
/// Returns `Ok` with a vector of loaded `KnownValue` instances, or an empty
/// vector if the directory doesn't exist. Returns `Err` only for I/O errors
/// that prevent directory traversal.
///
/// # Examples
///
/// ```rust,ignore
/// use known_values::load_from_directory;
/// use std::path::Path;
///
/// let values = load_from_directory(Path::new("/etc/known-values"))?;
/// for value in values {
///     println!("{}: {}", value.value(), value.name());
/// }
/// ```
pub fn load_from_directory(path: &Path) -> Result<Vec<KnownValue>, LoadError> {
    let mut values = Vec::new();

    // Return empty if directory doesn't exist or isn't a directory
    if !path.exists() || !path.is_dir() {
        return Ok(values);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        // Only process .json files
        if file_path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(&file_path)?;
            let registry: RegistryFile =
                serde_json::from_str(&content).map_err(|e| LoadError::Json {
                    file: file_path.clone(),
                    error: e,
                })?;

            for entry in registry.entries {
                values.push(KnownValue::new_with_name(
                    entry.codepoint,
                    entry.canonical_name,
                ));
            }
        }
    }

    Ok(values)
}

/// Loads known values from all directories in the given configuration.
///
/// Directories are processed in order. When multiple entries have the same
/// codepoint, values from later directories override values from earlier
/// directories.
///
/// This function is fault-tolerant: it will continue processing even if
/// some files fail to parse. Errors are collected in the returned
/// `LoadResult`.
///
/// # Arguments
///
/// * `config` - The directory configuration specifying search paths.
///
/// # Returns
///
/// A `LoadResult` containing the loaded values, processed files, and any
/// errors encountered.
///
/// # Examples
///
/// ```rust,ignore
/// use known_values::{DirectoryConfig, load_from_config};
///
/// let config = DirectoryConfig::default_only();
/// let result = load_from_config(&config);
///
/// println!("Loaded {} values from {} files",
///     result.values_count(),
///     result.files_processed.len());
///
/// if result.has_errors() {
///     for (path, error) in &result.errors {
///         eprintln!("Error loading {}: {}", path.display(), error);
///     }
/// }
/// ```
pub fn load_from_config(config: &DirectoryConfig) -> LoadResult {
    let mut result = LoadResult::default();

    for dir_path in config.paths() {
        match load_from_directory_tolerant(dir_path) {
            Ok((values, errors)) => {
                for value in values {
                    result.values.insert(value.value(), value);
                }
                if !errors.is_empty() {
                    result.errors.extend(errors);
                }
                result.files_processed.push(dir_path.clone());
            }
            Err(e) => {
                result.errors.push((dir_path.clone(), e));
            }
        }
    }

    result
}

/// Loads from a directory with tolerance for individual file failures.
fn load_from_directory_tolerant(
    path: &Path,
) -> Result<(Vec<KnownValue>, Vec<(PathBuf, LoadError)>), LoadError> {
    let mut values = Vec::new();
    let mut errors = Vec::new();

    if !path.exists() || !path.is_dir() {
        return Ok((values, errors));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.extension().map_or(false, |ext| ext == "json") {
            match load_single_file(&file_path) {
                Ok(file_values) => values.extend(file_values),
                Err(e) => errors.push((file_path, e)),
            }
        }
    }

    Ok((values, errors))
}

/// Loads known values from a single JSON file.
fn load_single_file(path: &Path) -> Result<Vec<KnownValue>, LoadError> {
    let content = fs::read_to_string(path)?;
    let registry: RegistryFile =
        serde_json::from_str(&content).map_err(|e| LoadError::Json {
            file: path.to_path_buf(),
            error: e,
        })?;

    Ok(registry
        .entries
        .into_iter()
        .map(|entry| KnownValue::new_with_name(entry.codepoint, entry.canonical_name))
        .collect())
}

// Global configuration state
static CUSTOM_CONFIG: Mutex<Option<DirectoryConfig>> = Mutex::new(None);
static CONFIG_LOCKED: AtomicBool = AtomicBool::new(false);

/// Error returned when configuration cannot be modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// Configuration was attempted after the global registry was initialized.
    AlreadyInitialized,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::AlreadyInitialized => {
                write!(
                    f,
                    "Cannot modify directory configuration after KNOWN_VALUES has been accessed"
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Sets custom directory configuration for known values loading.
///
/// This function must be called **before** the first access to `KNOWN_VALUES`.
/// Once `KNOWN_VALUES` is accessed, the configuration is locked and cannot
/// be changed.
///
/// # Arguments
///
/// * `config` - The directory configuration to use.
///
/// # Returns
///
/// Returns `Ok(())` if the configuration was set successfully, or
/// `Err(ConfigError::AlreadyInitialized)` if `KNOWN_VALUES` has already
/// been accessed.
///
/// # Examples
///
/// ```rust,ignore
/// use known_values::{set_directory_config, DirectoryConfig, KNOWN_VALUES};
///
/// // Set configuration before accessing KNOWN_VALUES
/// set_directory_config(DirectoryConfig::with_paths(vec![
///     "/custom/path".into(),
/// ])).expect("Configuration should succeed");
///
/// // Now access KNOWN_VALUES - it will use the custom configuration
/// let binding = KNOWN_VALUES.get();
/// ```
pub fn set_directory_config(config: DirectoryConfig) -> Result<(), ConfigError> {
    if CONFIG_LOCKED.load(Ordering::SeqCst) {
        return Err(ConfigError::AlreadyInitialized);
    }
    *CUSTOM_CONFIG.lock().unwrap() = Some(config);
    Ok(())
}

/// Adds additional search paths to the directory configuration.
///
/// This function must be called **before** the first access to `KNOWN_VALUES`.
/// Paths are added after any existing paths, so they will take precedence.
///
/// If no configuration has been set, this creates a new configuration with
/// the default directory and appends the new paths.
///
/// # Arguments
///
/// * `paths` - The paths to add to the configuration.
///
/// # Returns
///
/// Returns `Ok(())` if the paths were added successfully, or
/// `Err(ConfigError::AlreadyInitialized)` if `KNOWN_VALUES` has already
/// been accessed.
///
/// # Examples
///
/// ```rust,ignore
/// use known_values::add_search_paths;
///
/// // Add custom paths in addition to the default
/// add_search_paths(vec![
///     "/etc/known-values".into(),
///     "/usr/share/known-values".into(),
/// ]).expect("Should succeed before KNOWN_VALUES access");
/// ```
pub fn add_search_paths(paths: Vec<PathBuf>) -> Result<(), ConfigError> {
    if CONFIG_LOCKED.load(Ordering::SeqCst) {
        return Err(ConfigError::AlreadyInitialized);
    }
    let mut guard = CUSTOM_CONFIG.lock().unwrap();
    let config = guard.get_or_insert_with(DirectoryConfig::default_only);
    for path in paths {
        config.add_path(path);
    }
    Ok(())
}

/// Gets the current directory configuration, locking it for future modifications.
///
/// This is called internally during `KNOWN_VALUES` initialization.
pub(crate) fn get_and_lock_config() -> DirectoryConfig {
    CONFIG_LOCKED.store(true, Ordering::SeqCst);
    CUSTOM_CONFIG
        .lock()
        .unwrap()
        .take()
        .unwrap_or_else(DirectoryConfig::default_only)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_registry_json() {
        let json = r#"{
            "ontology": {"name": "test"},
            "entries": [
                {"codepoint": 9999, "canonical_name": "testValue", "type": "property"}
            ],
            "statistics": {}
        }"#;

        let registry: RegistryFile = serde_json::from_str(json).unwrap();
        assert_eq!(registry.entries.len(), 1);
        assert_eq!(registry.entries[0].codepoint, 9999);
        assert_eq!(registry.entries[0].canonical_name, "testValue");
    }

    #[test]
    fn test_parse_minimal_registry() {
        let json = r#"{"entries": [{"codepoint": 1, "canonical_name": "minimal"}]}"#;

        let registry: RegistryFile = serde_json::from_str(json).unwrap();
        assert_eq!(registry.entries.len(), 1);
        assert_eq!(registry.entries[0].codepoint, 1);
    }

    #[test]
    fn test_parse_full_entry() {
        let json = r#"{
            "entries": [{
                "codepoint": 100,
                "canonical_name": "fullEntry",
                "type": "class",
                "uri": "https://example.com/vocab#fullEntry",
                "description": "A complete entry with all fields"
            }]
        }"#;

        let registry: RegistryFile = serde_json::from_str(json).unwrap();
        let entry = &registry.entries[0];
        assert_eq!(entry.codepoint, 100);
        assert_eq!(entry.canonical_name, "fullEntry");
        assert_eq!(entry.entry_type.as_deref(), Some("class"));
        assert_eq!(
            entry.uri.as_deref(),
            Some("https://example.com/vocab#fullEntry")
        );
        assert!(entry.description.is_some());
    }

    #[test]
    fn test_directory_config_default() {
        let config = DirectoryConfig::default_only();
        assert_eq!(config.paths().len(), 1);
        assert!(config.paths()[0].ends_with(".known-values"));
    }

    #[test]
    fn test_directory_config_custom_paths() {
        let config =
            DirectoryConfig::with_paths(vec![PathBuf::from("/a"), PathBuf::from("/b")]);
        assert_eq!(config.paths().len(), 2);
        assert_eq!(config.paths()[0], PathBuf::from("/a"));
        assert_eq!(config.paths()[1], PathBuf::from("/b"));
    }

    #[test]
    fn test_directory_config_with_default() {
        let config =
            DirectoryConfig::with_paths_and_default(vec![PathBuf::from("/custom")]);
        assert_eq!(config.paths().len(), 2);
        assert_eq!(config.paths()[0], PathBuf::from("/custom"));
        assert!(config.paths()[1].ends_with(".known-values"));
    }

    #[test]
    fn test_load_from_nonexistent_directory() {
        let result = load_from_directory(Path::new("/nonexistent/path/12345"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_load_result_methods() {
        let mut result = LoadResult::default();
        assert_eq!(result.values_count(), 0);
        assert!(!result.has_errors());

        result
            .values
            .insert(1, KnownValue::new_with_name(1u64, "test".to_string()));
        assert_eq!(result.values_count(), 1);
    }
}
