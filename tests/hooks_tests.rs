//! Tests for hook implementations

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use tempfile::tempdir;
use rustyhook::hooks::{
    Hook,
    TrailingWhitespace, EndOfFileFixer, CheckYaml, CheckAddedLargeFiles,
    CheckMergeConflict, CheckJson, CheckToml, CheckXml, CheckCaseConflict,
    DetectPrivateKey
};

// Helper function to create a temporary file with content
fn create_temp_file(content: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_file.txt");
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    (dir, file_path)
}

#[test]
fn test_trailing_whitespace() {
    // Create a file with trailing whitespace
    let (dir, file_path) = create_temp_file("Hello world  \nThis is a test \n");

    // Run the hook
    let hook = TrailingWhitespace;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Check that the file was fixed
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Hello world\nThis is a test\n");

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_end_of_file_fixer() {
    // Create a file without a newline at the end
    let (dir, file_path) = create_temp_file("Hello world");

    // Run the hook
    let hook = EndOfFileFixer;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Check that the file was fixed
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Hello world\n");

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_yaml() {
    // Create a valid YAML file
    let (dir, file_path) = create_temp_file("key: value\nlist:\n  - item1\n  - item2\n");

    // Run the hook
    let hook = CheckYaml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Create an invalid YAML file
    let (dir2, file_path2) = create_temp_file("key: value\n  - invalid indentation\n  this is definitely invalid: : :\n");

    // Run the hook
    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir);
    drop(dir2);
}

#[test]
fn test_check_added_large_files() {
    // Create a file with a known size (2KB)
    let content = "x".repeat(2048);
    let (dir, file_path) = create_temp_file(&content);

    // Run the hook with a large max size
    let hook = CheckAddedLargeFiles::new(3);
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Run the hook with a small max size
    let hook = CheckAddedLargeFiles::new(1);
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_err());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_merge_conflict() {
    // Create a file without merge conflicts
    let (dir, file_path) = create_temp_file("Hello world\nThis is a test\n");

    // Run the hook
    let hook = CheckMergeConflict;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Create a file with merge conflicts
    let (dir2, file_path2) = create_temp_file("<<<<<<< HEAD\nHello world\n=======\nGoodbye world\n>>>>>>> branch\n");

    // Run the hook
    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir);
    drop(dir2);
}

#[test]
fn test_check_json() {
    // Create a valid JSON file
    let (dir, file_path) = create_temp_file("{\"key\": \"value\", \"list\": [1, 2, 3]}");

    // Run the hook
    let hook = CheckJson;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Create an invalid JSON file
    let (dir2, file_path2) = create_temp_file("{\"key\": \"value\", \"list\": [1, 2, 3}");

    // Run the hook
    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir);
    drop(dir2);
}

#[test]
fn test_check_toml() {
    // Create a valid TOML file
    let (dir, file_path) = create_temp_file("key = \"value\"\n[section]\nitem = 123\n");

    // Run the hook
    let hook = CheckToml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Create an invalid TOML file (missing key-value pairs)
    let (dir2, file_path2) = create_temp_file("[section]\n# just a comment\n");

    // Run the hook
    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir);
    drop(dir2);
}

#[test]
fn test_check_xml() {
    // Create a valid XML file
    let (dir, file_path) = create_temp_file("<root><item>value</item></root>");

    // Run the hook
    let hook = CheckXml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Create an invalid XML file with mismatched tags
    let (dir2, file_path2) = create_temp_file("<root><item>value</item");

    // Run the hook
    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir);
    drop(dir2);
}

#[test]
fn test_check_case_conflict() {
    // Create a temporary directory
    let dir = tempdir().unwrap();

    // Create files without case conflicts
    let file_path1 = dir.path().join("file1.txt");
    let file_path2 = dir.path().join("file2.txt");
    fs::write(&file_path1, "content").unwrap();
    fs::write(&file_path2, "content").unwrap();

    // Run the hook
    let hook = CheckCaseConflict;
    let result = hook.run(&[file_path1.clone(), file_path2.clone()]);
    assert!(result.is_ok());

    // Create files with case conflicts
    let file_path3 = dir.path().join("File1.txt");
    fs::write(&file_path3, "content").unwrap();

    // Run the hook
    let result = hook.run(&[file_path1.clone(), file_path2.clone(), file_path3.clone()]);
    assert!(result.is_err());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_detect_private_key() {
    // Create a file without private keys
    let (dir, file_path) = create_temp_file("Hello world\nThis is a test\n");

    // Run the hook
    let hook = DetectPrivateKey;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Create a file with a private key
    let (dir2, file_path2) = create_temp_file("-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA1JrNWQ3yTd4\n-----END RSA PRIVATE KEY-----\n");

    // Run the hook
    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir);
    drop(dir2);
}
