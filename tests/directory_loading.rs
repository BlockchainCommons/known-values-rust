//! Integration tests for the directory-loading feature.

#[cfg(feature = "directory-loading")]
mod tests {
    use std::path::Path;

    use known_values::{
        DirectoryConfig, IS_A, KNOWN_VALUES, KnownValuesStore, NOTE,
    };
    use tempfile::TempDir;

    #[test]
    fn test_global_registry_still_works() {
        // Verify KNOWN_VALUES still works with feature enabled
        let binding = KNOWN_VALUES.get();
        let store = binding.as_ref().unwrap();

        // Hardcoded values should still be present
        let is_a = store.known_value_named("isA");
        assert!(is_a.is_some());
        assert_eq!(is_a.unwrap().value(), 1);
    }

    #[test]
    fn test_load_from_temp_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_registry.json");

        let json = r#"{
            "entries": [
                {"codepoint": 99999, "name": "integrationTestValue"}
            ]
        }"#;
        std::fs::write(&file_path, json).unwrap();

        let mut store = KnownValuesStore::new([IS_A, NOTE]);
        let count = store.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 1);

        let loaded = store.known_value_named("integrationTestValue");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().value(), 99999);

        // Original values should still be present
        assert!(store.known_value_named("isA").is_some());
        assert!(store.known_value_named("note").is_some());
    }

    #[test]
    fn test_override_hardcoded_value() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("override.json");

        // Override IS_A (codepoint 1) with a custom name
        let json = r#"{
            "entries": [
                {"codepoint": 1, "name": "overriddenIsA"}
            ]
        }"#;
        std::fs::write(&file_path, json).unwrap();

        let mut store = KnownValuesStore::new([IS_A]);
        store.load_from_directory(temp_dir.path()).unwrap();

        // The original "isA" name should be gone (replaced)
        let original = store.known_value_named("isA");
        assert!(original.is_none());

        // The new name should work
        let overridden = store.known_value_named("overriddenIsA");
        assert!(overridden.is_some());
        assert_eq!(overridden.unwrap().value(), 1);
    }

    #[test]
    fn test_multiple_files_in_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple JSON files
        let file1 = temp_dir.path().join("registry1.json");
        let file2 = temp_dir.path().join("registry2.json");

        std::fs::write(
            &file1,
            r#"{"entries": [{"codepoint": 10001, "name": "valueOne"}]}"#,
        )
        .unwrap();
        std::fs::write(
            &file2,
            r#"{"entries": [{"codepoint": 10002, "name": "valueTwo"}]}"#,
        )
        .unwrap();

        let mut store = KnownValuesStore::default();
        let count = store.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 2);
        assert!(store.known_value_named("valueOne").is_some());
        assert!(store.known_value_named("valueTwo").is_some());
    }

    #[test]
    fn test_directory_config_custom_paths() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // First directory has value A
        std::fs::write(
            temp_dir1.path().join("a.json"),
            r#"{"entries": [{"codepoint": 20001, "name": "fromDirOne"}]}"#,
        )
        .unwrap();

        // Second directory has value B
        std::fs::write(
            temp_dir2.path().join("b.json"),
            r#"{"entries": [{"codepoint": 20002, "name": "fromDirTwo"}]}"#,
        )
        .unwrap();

        let config = DirectoryConfig::with_paths(vec![
            temp_dir1.path().to_path_buf(),
            temp_dir2.path().to_path_buf(),
        ]);

        let mut store = KnownValuesStore::default();
        let result = store.load_from_config(&config);

        assert_eq!(result.values_count(), 2);
        assert!(store.known_value_named("fromDirOne").is_some());
        assert!(store.known_value_named("fromDirTwo").is_some());
    }

    #[test]
    fn test_later_directory_overrides_earlier() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // Both directories have same codepoint with different names
        std::fs::write(
            temp_dir1.path().join("first.json"),
            r#"{"entries": [{"codepoint": 30000, "name": "firstVersion"}]}"#,
        )
        .unwrap();

        std::fs::write(
            temp_dir2.path().join("second.json"),
            r#"{"entries": [{"codepoint": 30000, "name": "secondVersion"}]}"#,
        )
        .unwrap();

        // Process dir1 first, then dir2
        let config = DirectoryConfig::with_paths(vec![
            temp_dir1.path().to_path_buf(),
            temp_dir2.path().to_path_buf(),
        ]);

        let mut store = KnownValuesStore::default();
        store.load_from_config(&config);

        // Second directory should win (later in list)
        let value = store.known_value_named("secondVersion");
        assert!(value.is_some());
        assert_eq!(value.unwrap().value(), 30000);

        // First name should be gone
        assert!(store.known_value_named("firstVersion").is_none());
    }

    #[test]
    fn test_nonexistent_directory_is_ok() {
        let mut store = KnownValuesStore::default();
        let result =
            store.load_from_directory(Path::new("/nonexistent/path/12345"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_invalid_json_is_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.json");

        std::fs::write(&file_path, "{ this is not valid json }").unwrap();

        let mut store = KnownValuesStore::default();
        let result = store.load_from_directory(temp_dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_tolerant_loading_continues_on_error() {
        let temp_dir = TempDir::new().unwrap();

        // One valid file
        std::fs::write(
            temp_dir.path().join("valid.json"),
            r#"{"entries": [{"codepoint": 40001, "name": "validValue"}]}"#,
        )
        .unwrap();

        // One invalid file
        std::fs::write(
            temp_dir.path().join("invalid.json"),
            "{ invalid json }",
        )
        .unwrap();

        let config =
            DirectoryConfig::with_paths(vec![temp_dir.path().to_path_buf()]);
        let result = known_values::load_from_config(&config);

        // Should have loaded the valid value
        assert!(result.values.contains_key(&40001));

        // Should have recorded the error
        assert!(result.has_errors());
    }

    #[test]
    fn test_full_registry_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("full_format.json");

        // Use the full format from BlockchainCommons research repo
        let json = r#"{
            "ontology": {
                "name": "test_registry",
                "source_url": "https://example.com",
                "start_code_point": 50000,
                "processing_strategy": "test"
            },
            "generated": {
                "tool": "test"
            },
            "entries": [
                {
                    "codepoint": 50001,
                    "name": "fullFormatValue",
                    "type": "property",
                    "uri": "https://example.com/vocab#fullFormatValue",
                    "description": "A value in full format"
                },
                {
                    "codepoint": 50002,
                    "name": "anotherValue",
                    "type": "class"
                }
            ],
            "statistics": {
                "total_entries": 2
            }
        }"#;
        std::fs::write(&file_path, json).unwrap();

        let mut store = KnownValuesStore::default();
        let count = store.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 2);
        assert!(store.known_value_named("fullFormatValue").is_some());
        assert!(store.known_value_named("anotherValue").is_some());
    }

    #[test]
    fn test_load_result_methods() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("test.json"),
            r#"{"entries": [
                {"codepoint": 60001, "name": "resultTest1"},
                {"codepoint": 60002, "name": "resultTest2"}
            ]}"#,
        )
        .unwrap();

        let config =
            DirectoryConfig::with_paths(vec![temp_dir.path().to_path_buf()]);
        let result = known_values::load_from_config(&config);

        assert_eq!(result.values_count(), 2);
        assert!(!result.has_errors());
        assert_eq!(result.files_processed.len(), 1);

        // Test iteration
        let values: Vec<_> = result.values_iter().collect();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_empty_entries_array() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("empty.json"),
            r#"{"entries": []}"#,
        )
        .unwrap();

        let mut store = KnownValuesStore::default();
        let count = store.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_non_json_files_ignored() {
        let temp_dir = TempDir::new().unwrap();

        // JSON file should be loaded
        std::fs::write(
            temp_dir.path().join("valid.json"),
            r#"{"entries": [{"codepoint": 70001, "name": "jsonValue"}]}"#,
        )
        .unwrap();

        // Non-JSON files should be ignored
        std::fs::write(temp_dir.path().join("readme.txt"), "Some text")
            .unwrap();
        std::fs::write(temp_dir.path().join("data.xml"), "<xml/>").unwrap();

        let mut store = KnownValuesStore::default();
        let count = store.load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 1);
        assert!(store.known_value_named("jsonValue").is_some());
    }
}
