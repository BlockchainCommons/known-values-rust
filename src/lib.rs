//! Known Values: A compact, deterministic representation for ontological concepts.
//!
//! This crate implements the [Blockchain Commons Known Values specification][bcr],
//! providing a compact way to represent ontological concepts using 64-bit unsigned
//! integers with optional human-readable names.
//!
//! # Basic Usage
//!
//! ```rust
//! use known_values::{KnownValue, KnownValuesStore, IS_A, NOTE};
//!
//! // Use predefined constants
//! assert_eq!(IS_A.value(), 1);
//! assert_eq!(IS_A.name(), "isA");
//!
//! // Create custom known values
//! let custom = KnownValue::new_with_name(1000u64, "myCustomValue".to_string());
//! assert_eq!(custom.value(), 1000);
//!
//! // Use a store for bidirectional lookup
//! let store = KnownValuesStore::new([IS_A, NOTE]);
//! assert_eq!(store.known_value_named("isA").unwrap().value(), 1);
//! ```
//!
//! # Directory Loading Feature
//!
//! When the `directory-loading` feature is enabled (default), this crate can
//! load additional known values from JSON registry files.
//!
//! ## Default Behavior
//!
//! On first access to [`KNOWN_VALUES`], the crate automatically:
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
//! Configure search paths before first access (requires `directory-loading` feature):
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
//!
//! [bcr]: https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2023-002-known-value.md

mod known_value;
pub use known_value::KnownValue;

mod known_value_store;
pub use known_value_store::KnownValuesStore;

mod known_values_registry;
pub use known_values_registry::*;

#[cfg(feature = "directory-loading")]
mod directory_loader;

#[cfg(feature = "directory-loading")]
pub use directory_loader::{
    add_search_paths, load_from_config, load_from_directory, set_directory_config,
    ConfigError, DirectoryConfig, LoadError, LoadResult, RegistryEntry, RegistryFile,
};
