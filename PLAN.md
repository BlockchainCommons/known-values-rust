# Implementation Plan: Directory-Based Known Values Loading

## Overview

Add a feature-gated capability to load known values from JSON registry files in configurable directories. Values loaded from JSON files supersede hardcoded constants, enabling users to extend or override the default registry without recompiling.

## JSON Registry Format

The research repo uses this structure (files like `0_blockchain_commons_registry.json`):

```json
{
  "ontology": {
    "name": "blockchain_commons_registry",
    "source_url": "...",
    "start_code_point": 0,
    "processing_strategy": "..."
  },
  "generated": { "tool": "..." },
  "entries": [
    {
      "codepoint": 1,
      "canonical_name": "isA",
      "type": "property",
      "uri": "...",
      "description": "..."
    }
  ],
  "statistics": { ... }
}
```

---

## Step 1: Add Feature Gate to `Cargo.toml`

```toml
[features]
default = ["directory-loading"]
directory-loading = ["dep:serde", "dep:serde_json", "dep:dirs"]

[dependencies]
# Existing dependencies...
bc-components = "^0.31.0"
dcbor = { version = "^0.25.0", features = ["multithreaded"] }
paste = "^1.0.12"

# Optional dependencies for directory loading
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
dirs = { version = "5.0", optional = true }
```

**Rationale**:
- Feature is `on` by default
- Users can opt-out with `default-features = false`
- `serde`/`serde_json` for JSON parsing
- `dirs` for cross-platform home directory resolution (`~/.known-values/`)

---

## Step 2: Create New Module `src/directory_loader.rs`

### 2.1 JSON Deserialization Structures

```rust
#[cfg(feature = "directory-loading")]
mod directory_loader {
    use serde::Deserialize;
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::io;

    use crate::{KnownValue, KnownValuesStore};

    /// A single entry in a known values JSON registry file.
    #[derive(Debug, Deserialize)]
    pub struct RegistryEntry {
        pub codepoint: u64,
        pub canonical_name: String,
        #[serde(rename = "type")]
        pub entry_type: Option<String>,
        pub uri: Option<String>,
        pub description: Option<String>,
    }

    /// Root structure of a known values JSON registry file.
    #[derive(Debug, Deserialize)]
    pub struct RegistryFile {
        pub ontology: Option<OntologyInfo>,
        pub entries: Vec<RegistryEntry>,
        // Other fields ignored
    }

    #[derive(Debug, Deserialize)]
    pub struct OntologyInfo {
        pub name: Option<String>,
        pub source_url: Option<String>,
        pub start_code_point: Option<u64>,
    }
}
```

### 2.2 Directory Configuration

```rust
/// Configuration for loading known values from directories.
#[derive(Debug, Clone)]
pub struct DirectoryConfig {
    /// Search paths in priority order (later paths override earlier)
    paths: Vec<PathBuf>,
}

impl DirectoryConfig {
    /// Creates configuration with only the default directory (~/.known-values/).
    pub fn default_only() -> Self {
        Self {
            paths: vec![Self::default_directory()],
        }
    }

    /// Creates configuration with custom paths (processed in order).
    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }

    /// Creates configuration with custom paths followed by the default.
    pub fn with_paths_and_default(mut paths: Vec<PathBuf>) -> Self {
        paths.push(Self::default_directory());
        Self { paths }
    }

    /// Returns the default directory: ~/.known-values/
    pub fn default_directory() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".known-values")
    }

    /// Returns the configured search paths.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

impl Default for DirectoryConfig {
    fn default() -> Self {
        Self::default_only()
    }
}
```

### 2.3 Loading Functions

```rust
/// Errors that can occur when loading known values from directories.
#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {}", e),
            LoadError::Json(e) => write!(f, "JSON parse error: {}", e),
        }
    }
}

impl std::error::Error for LoadError {}

/// Result of loading operations.
pub struct LoadResult {
    /// Number of values loaded
    pub values_loaded: usize,
    /// Files processed
    pub files_processed: Vec<PathBuf>,
    /// Errors encountered (non-fatal)
    pub errors: Vec<(PathBuf, LoadError)>,
}

/// Loads all JSON registry files from a single directory.
pub fn load_from_directory(path: &Path) -> Result<Vec<KnownValue>, LoadError> {
    let mut values = Vec::new();

    if !path.exists() || !path.is_dir() {
        return Ok(values); // Empty if directory doesn't exist
    }

    for entry in fs::read_dir(path).map_err(LoadError::Io)? {
        let entry = entry.map_err(LoadError::Io)?;
        let file_path = entry.path();

        if file_path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(&file_path).map_err(LoadError::Io)?;
            let registry: RegistryFile = serde_json::from_str(&content)
                .map_err(LoadError::Json)?;

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

/// Loads known values from all configured directories.
///
/// Directories are processed in order; values from later directories
/// override values from earlier ones (by codepoint).
pub fn load_from_config(config: &DirectoryConfig) -> LoadResult {
    let mut result = LoadResult {
        values_loaded: 0,
        files_processed: Vec::new(),
        errors: Vec::new(),
    };

    let mut all_values: std::collections::HashMap<u64, KnownValue> =
        std::collections::HashMap::new();

    for dir_path in config.paths() {
        match load_from_directory(dir_path) {
            Ok(values) => {
                for value in values {
                    all_values.insert(value.value(), value);
                    result.values_loaded += 1;
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
```

---

## Step 3: Modify `LazyKnownValues` Initialization

Update `src/known_values_registry.rs` to conditionally load from directories:

```rust
impl LazyKnownValues {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<KnownValuesStore>> {
        self.init.call_once(|| {
            // Start with hardcoded values
            let mut store = KnownValuesStore::new([
                UNIT, IS_A, ID, /* ... all existing constants ... */
            ]);

            // When feature is enabled, load from directories and override
            #[cfg(feature = "directory-loading")]
            {
                use crate::directory_loader::{DirectoryConfig, load_from_config};

                let config = DirectoryConfig::default();
                let result = load_from_config(&config);

                // Insert loaded values (overrides hardcoded on collision)
                for value in result.values_loaded_iter() {
                    store.insert(value);
                }
            }

            *self.data.lock().unwrap() = Some(store);
        });
        self.data.lock().unwrap()
    }
}
```

---

## Step 4: Add Configuration API for Custom Paths

Add methods to allow runtime configuration before first access:

```rust
#[cfg(feature = "directory-loading")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "directory-loading")]
static CUSTOM_CONFIG: Mutex<Option<DirectoryConfig>> = Mutex::new(None);

#[cfg(feature = "directory-loading")]
static CONFIG_LOCKED: AtomicBool = AtomicBool::new(false);

/// Sets custom directory configuration for known values loading.
///
/// Must be called BEFORE first access to `KNOWN_VALUES`.
/// Returns `Err` if called after initialization.
#[cfg(feature = "directory-loading")]
pub fn set_directory_config(config: DirectoryConfig) -> Result<(), ConfigError> {
    if CONFIG_LOCKED.load(Ordering::SeqCst) {
        return Err(ConfigError::AlreadyInitialized);
    }
    *CUSTOM_CONFIG.lock().unwrap() = Some(config);
    Ok(())
}

/// Adds additional search paths to the default configuration.
///
/// Must be called BEFORE first access to `KNOWN_VALUES`.
#[cfg(feature = "directory-loading")]
pub fn add_search_paths(paths: Vec<PathBuf>) -> Result<(), ConfigError> {
    if CONFIG_LOCKED.load(Ordering::SeqCst) {
        return Err(ConfigError::AlreadyInitialized);
    }
    let mut guard = CUSTOM_CONFIG.lock().unwrap();
    let config = guard.get_or_insert_with(DirectoryConfig::default);
    for path in paths {
        config.paths.push(path);
    }
    Ok(())
}

#[cfg(feature = "directory-loading")]
#[derive(Debug, Clone)]
pub enum ConfigError {
    AlreadyInitialized,
}
```

---

## Step 5: Update Module Structure (`src/lib.rs`)

```rust
mod known_value;
mod known_value_store;
mod known_values_registry;

#[cfg(feature = "directory-loading")]
mod directory_loader;

pub use known_value::KnownValue;
pub use known_value_store::KnownValuesStore;
pub use known_values_registry::*;

#[cfg(feature = "directory-loading")]
pub use directory_loader::{
    DirectoryConfig,
    LoadError,
    LoadResult,
    load_from_directory,
    load_from_config,
    set_directory_config,
    add_search_paths,
    ConfigError,
};
```

---

## Step 6: Add Extended `KnownValuesStore` Methods

Add convenience methods for manual loading:

```rust
impl KnownValuesStore {
    /// Loads and inserts known values from a directory.
    /// Values from JSON override existing values with the same codepoint.
    #[cfg(feature = "directory-loading")]
    pub fn load_from_directory(&mut self, path: &Path) -> Result<usize, LoadError> {
        let values = crate::directory_loader::load_from_directory(path)?;
        let count = values.len();
        for value in values {
            self.insert(value);
        }
        Ok(count)
    }

    /// Loads known values from multiple directories using configuration.
    #[cfg(feature = "directory-loading")]
    pub fn load_from_config(&mut self, config: &DirectoryConfig) -> LoadResult {
        let result = crate::directory_loader::load_from_config(config);
        // Values already collected in result, insert them
        for (_, values) in &result.values_by_directory {
            for value in values {
                self.insert(value.clone());
            }
        }
        result
    }
}
```

---

## Step 7: Testing Strategy

### Unit Tests (`src/directory_loader.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

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
    fn test_load_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_registry.json");

        let json = r#"{"entries": [{"codepoint": 8888, "canonical_name": "dirTest"}]}"#;
        std::fs::write(&file_path, json).unwrap();

        let values = load_from_directory(temp_dir.path()).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].value(), 8888);
        assert_eq!(values[0].name(), "dirTest");
    }

    #[test]
    fn test_override_hardcoded_value() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("override.json");

        // Override IS_A (codepoint 1) with custom name
        let json = r#"{"entries": [{"codepoint": 1, "canonical_name": "customIsA"}]}"#;
        std::fs::write(&file_path, json).unwrap();

        let mut store = KnownValuesStore::new([crate::IS_A]);
        store.load_from_directory(temp_dir.path()).unwrap();

        // Verify override took effect
        let is_a = store.known_value_named("customIsA").unwrap();
        assert_eq!(is_a.value(), 1);
    }

    #[test]
    fn test_missing_directory_returns_empty() {
        let values = load_from_directory(Path::new("/nonexistent/path")).unwrap();
        assert!(values.is_empty());
    }
}
```

### Integration Tests (`tests/directory_loading.rs`)

```rust
#[cfg(feature = "directory-loading")]
mod tests {
    use known_values::*;

    #[test]
    fn test_global_registry_with_directory_loading() {
        // Verify KNOWN_VALUES still works with feature enabled
        let binding = KNOWN_VALUES.get();
        let store = binding.as_ref().unwrap();

        // Hardcoded values should still be present
        assert!(store.known_value_named("isA").is_some());
    }
}
```

---

## Step 8: Documentation

### Module-Level Documentation

```rust
//! # Directory Loading Feature
//!
//! When the `directory-loading` feature is enabled (default), this crate can
//! load additional known values from JSON registry files.
//!
//! ## Default Behavior
//!
//! On first access to `KNOWN_VALUES`, the crate automatically:
//! 1. Initializes hardcoded known values from the registry
//! 2. Scans `~/.known-values/` for JSON files
//! 3. Loads entries from any `*.json` files found
//! 4. Overrides hardcoded values if codepoints collide
//!
//! ## JSON File Format
//!
//! Registry files should follow the BlockchainCommons format:
//!
//! ```json
//! {
//!   "entries": [
//!     {"codepoint": 1000, "canonical_name": "myValue", "type": "property"}
//!   ]
//! }
//! ```
//!
//! ## Custom Configuration
//!
//! Configure search paths before first access:
//!
//! ```rust,ignore
//! use known_values::{set_directory_config, DirectoryConfig};
//!
//! // Use only custom paths
//! set_directory_config(DirectoryConfig::with_paths(vec![
//!     "/etc/known-values".into(),
//!     "/usr/share/known-values".into(),
//! ])).unwrap();
//! ```
//!
//! ## Disabling Directory Loading
//!
//! To disable at compile time:
//!
//! ```toml
//! [dependencies]
//! known-values = { version = "0.15", default-features = false }
//! ```
```

---

## File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` | Modify | Add `directory-loading` feature (default), add `serde`, `serde_json`, `dirs` deps |
| `src/lib.rs` | Modify | Conditionally export `directory_loader` module |
| `src/directory_loader.rs` | **Create** | JSON parsing, directory scanning, configuration |
| `src/known_values_registry.rs` | Modify | Enhance `LazyKnownValues::get()` to load from dirs |
| `src/known_value_store.rs` | Modify | Add `load_from_directory()` and `load_from_config()` methods |
| `tests/directory_loading.rs` | **Create** | Integration tests |

---

## API Summary

### New Public Types (feature-gated)

- `DirectoryConfig` - Configuration for search paths
- `LoadError` - Error type for loading operations
- `LoadResult` - Result with stats and error details
- `ConfigError` - Error for late configuration attempts

### New Public Functions (feature-gated)

- `load_from_directory(path) -> Result<Vec<KnownValue>, LoadError>`
- `load_from_config(config) -> LoadResult`
- `set_directory_config(config) -> Result<(), ConfigError>`
- `add_search_paths(paths) -> Result<(), ConfigError>`

### Enhanced Methods (feature-gated)

- `KnownValuesStore::load_from_directory(&mut self, path) -> Result<usize, LoadError>`
- `KnownValuesStore::load_from_config(&mut self, config) -> LoadResult`

### Unchanged API

- All existing constants (`IS_A`, `NOTE`, etc.)
- `KnownValue` struct and methods
- `KnownValuesStore::new()`, `insert()`, `known_value_named()`, etc.
- `KNOWN_VALUES` global (enhanced initialization when feature active)

---

## Implementation Order

1. Add dependencies and feature gate to `Cargo.toml`
2. Create `src/directory_loader.rs` with JSON structures and loading functions
3. Update `src/lib.rs` to conditionally export the module
4. Modify `LazyKnownValues::get()` in `src/known_values_registry.rs`
5. Add convenience methods to `KnownValuesStore`
6. Write unit tests
7. Write integration tests
8. Update documentation
