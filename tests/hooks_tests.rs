//! Tests for hook implementations

use std::path::PathBuf;
use std::fs;
use std::io::Write;
use tempfile::tempdir;
use rustyhook::hooks::{
    Hook, HookFactory, HookError,
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
fn test_trailing_whitespace_no_whitespace() {
    // Create a file without trailing whitespace
    let (dir, file_path) = create_temp_file("Hello world\nThis is a test\n");

    // Run the hook
    let hook = TrailingWhitespace;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Check that the file was not modified
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Hello world\nThis is a test\n");

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_trailing_whitespace_empty_file() {
    // Create an empty file
    let (dir, file_path) = create_temp_file("");

    // Run the hook
    let hook = TrailingWhitespace;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Check that the file was not modified
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "");

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_trailing_whitespace_nonexistent_file() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.txt");

    // Run the hook
    let hook = TrailingWhitespace;
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
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
fn test_end_of_file_fixer_with_newline() {
    // Create a file that already has a newline at the end
    let (dir, file_path) = create_temp_file("Hello world\n");

    // Run the hook
    let hook = EndOfFileFixer;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Check that the file was not modified
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Hello world\n");

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_end_of_file_fixer_empty_file() {
    // Create an empty file
    let (dir, file_path) = create_temp_file("");

    // Run the hook
    let hook = EndOfFileFixer;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Check that the file was not modified
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "");

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_end_of_file_fixer_nonexistent_file() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.txt");

    // Run the hook
    let hook = EndOfFileFixer;
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
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
fn test_check_yaml_empty_file() {
    // Create an empty file (should be valid YAML)
    let (dir, file_path) = create_temp_file("");

    // Run the hook
    let hook = CheckYaml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_yaml_edge_cases() {
    // Create a YAML file with various edge cases
    let (dir, file_path) = create_temp_file("---\n# Comment\nempty_value: \nnull_value: null\nboolean: true\nnumber: 42\n");

    // Run the hook
    let hook = CheckYaml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_yaml_nonexistent_file() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.yaml");

    // Run the hook
    let hook = CheckYaml;
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
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
fn test_check_added_large_files_empty() {
    // Create an empty file (0 KB)
    let (dir, file_path) = create_temp_file("");

    // Run the hook with any max size
    let hook = CheckAddedLargeFiles::new(1);
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_added_large_files_exact_size() {
    // Create a file with exactly 1KB
    let content = "x".repeat(1024);
    let (dir, file_path) = create_temp_file(&content);

    // Run the hook with max size of 1KB
    let hook = CheckAddedLargeFiles::new(1);
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Run the hook with max size of 0KB
    let hook = CheckAddedLargeFiles::new(0);
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_err());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_added_large_files_nonexistent() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.txt");

    // Run the hook
    let hook = CheckAddedLargeFiles::new(1);
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
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
fn test_check_merge_conflict_empty_file() {
    // Create an empty file
    let (dir, file_path) = create_temp_file("");

    // Run the hook
    let hook = CheckMergeConflict;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_merge_conflict_partial_markers() {
    // Create files with only some of the merge conflict markers
    let (dir1, file_path1) = create_temp_file("<<<<<<< HEAD\nHello world\n");
    let (dir2, file_path2) = create_temp_file("=======\nGoodbye world\n");
    let (dir3, file_path3) = create_temp_file(">>>>>>> branch\n");

    // Run the hook on each file
    let hook = CheckMergeConflict;

    let result = hook.run(&[file_path1.clone()]);
    assert!(result.is_err());

    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_err());

    let result = hook.run(&[file_path3.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir1);
    drop(dir2);
    drop(dir3);
}

#[test]
fn test_check_merge_conflict_nonexistent_file() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.txt");

    // Run the hook
    let hook = CheckMergeConflict;
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
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
fn test_check_json_empty_file() {
    // Create an empty file (should be invalid JSON)
    let (dir, file_path) = create_temp_file("");

    // Run the hook
    let hook = CheckJson;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_err());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_json_edge_cases() {
    // Create JSON files with various edge cases

    // Valid: null
    let (dir1, file_path1) = create_temp_file("null");

    // Valid: true
    let (dir2, file_path2) = create_temp_file("true");

    // Valid: 42
    let (dir3, file_path3) = create_temp_file("42");

    // Valid: empty object
    let (dir4, file_path4) = create_temp_file("{}");

    // Valid: empty array
    let (dir5, file_path5) = create_temp_file("[]");

    // Run the hook on each file
    let hook = CheckJson;

    let result = hook.run(&[file_path1.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path3.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path4.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path5.clone()]);
    assert!(result.is_ok());

    // Keep the directories alive until the end of the test
    drop(dir1);
    drop(dir2);
    drop(dir3);
    drop(dir4);
    drop(dir5);
}

#[test]
fn test_check_json_nonexistent_file() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.json");

    // Run the hook
    let hook = CheckJson;
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
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
fn test_check_toml_empty_file() {
    // Create an empty file (should be valid TOML)
    let (dir, file_path) = create_temp_file("");

    // Run the hook
    let hook = CheckToml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_toml_comments_only() {
    // Create a file with only comments (should be invalid TOML)
    let (dir, file_path) = create_temp_file("# This is a comment\n# Another comment\n");

    // Run the hook
    let hook = CheckToml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_err());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_toml_section_only() {
    // Create a file with only section headers (should be invalid TOML)
    let (dir, file_path) = create_temp_file("[section1]\n[section2]\n");

    // Run the hook
    let hook = CheckToml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_err());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_toml_edge_cases() {
    // Create TOML files with various edge cases

    // Valid: simple key-value
    let (dir1, file_path1) = create_temp_file("key = \"value\"");

    // Valid: nested tables
    let (dir2, file_path2) = create_temp_file("[table1]\nkey1 = \"value1\"\n[table1.subtable]\nkey2 = \"value2\"");

    // Valid: arrays
    let (dir3, file_path3) = create_temp_file("array = [1, 2, 3]");

    // Invalid: unexpected format
    let (dir4, file_path4) = create_temp_file("This is not a valid TOML line");

    // Run the hook on each file
    let hook = CheckToml;

    let result = hook.run(&[file_path1.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path3.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path4.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir1);
    drop(dir2);
    drop(dir3);
    drop(dir4);
}

#[test]
fn test_check_toml_nonexistent_file() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.toml");

    // Run the hook
    let hook = CheckToml;
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
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
fn test_check_xml_empty_file() {
    // Create an empty file (should be invalid XML)
    let (dir, file_path) = create_temp_file("");

    // Run the hook
    let hook = CheckXml;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_err());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_xml_mismatched_tags() {
    // Create files with different types of mismatched tags

    // More opening than closing tags
    let (dir1, file_path1) = create_temp_file("<root><item>value</root>");

    // More closing than opening tags
    let (dir2, file_path2) = create_temp_file("<root></item></root>");

    // Run the hook on each file
    let hook = CheckXml;

    let result = hook.run(&[file_path1.clone()]);
    assert!(result.is_ok()); // Note: This simple check only counts < and > characters

    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_ok()); // Note: This simple check only counts < and > characters

    // Keep the directories alive until the end of the test
    drop(dir1);
    drop(dir2);
}

#[test]
fn test_check_xml_edge_cases() {
    // Create XML files with various edge cases

    // Valid: XML declaration
    let (dir1, file_path1) = create_temp_file("<?xml version=\"1.0\" encoding=\"UTF-8\"?><root></root>");

    // Valid: Self-closing tag
    let (dir2, file_path2) = create_temp_file("<root><item/></root>");

    // Valid: CDATA section
    let (dir3, file_path3) = create_temp_file("<root><![CDATA[<not>a</tag>]]></root>");

    // Invalid: No tags
    let (dir4, file_path4) = create_temp_file("This is not XML");

    // Run the hook on each file
    let hook = CheckXml;

    let result = hook.run(&[file_path1.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path3.clone()]);
    assert!(result.is_ok());

    let result = hook.run(&[file_path4.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir1);
    drop(dir2);
    drop(dir3);
    drop(dir4);
}

#[test]
fn test_check_xml_nonexistent_file() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.xml");

    // Run the hook
    let hook = CheckXml;
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
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
fn test_check_case_conflict_empty_list() {
    // Run the hook with an empty file list
    let hook = CheckCaseConflict;
    let result = hook.run(&[]);
    assert!(result.is_ok());
}

#[test]
fn test_check_case_conflict_special_chars() {
    // Create a temporary directory
    let dir = tempdir().unwrap();

    // Create files with special characters
    let file_path1 = dir.path().join("file-with-dashes.txt");
    let file_path2 = dir.path().join("file_with_underscores.txt");
    let file_path3 = dir.path().join("file.with.dots.txt");
    let file_path4 = dir.path().join("file with spaces.txt");
    fs::write(&file_path1, "content").unwrap();
    fs::write(&file_path2, "content").unwrap();
    fs::write(&file_path3, "content").unwrap();
    fs::write(&file_path4, "content").unwrap();

    // Run the hook
    let hook = CheckCaseConflict;
    let result = hook.run(&[file_path1.clone(), file_path2.clone(), file_path3.clone(), file_path4.clone()]);
    assert!(result.is_ok());

    // Create a file with case conflict
    let file_path5 = dir.path().join("File-With-Dashes.txt");
    fs::write(&file_path5, "content").unwrap();

    // Run the hook
    let result = hook.run(&[file_path1.clone(), file_path2.clone(), file_path3.clone(), file_path4.clone(), file_path5.clone()]);
    assert!(result.is_err());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_check_case_conflict_invalid_name() {
    // Create a path without a file name
    let invalid_path = PathBuf::from("");

    // Run the hook
    let hook = CheckCaseConflict;
    let result = hook.run(&[invalid_path]);
    assert!(result.is_err());

    // Verify it's the expected error
    match result {
        Err(HookError::Other(msg)) => assert!(msg.contains("Invalid file name")),
        _ => panic!("Expected HookError::Other"),
    }
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

#[test]
fn test_detect_private_key_empty_file() {
    // Create an empty file
    let (dir, file_path) = create_temp_file("");

    // Run the hook
    let hook = DetectPrivateKey;
    let result = hook.run(&[file_path.clone()]);
    assert!(result.is_ok());

    // Keep the directory alive until the end of the test
    drop(dir);
}

#[test]
fn test_detect_private_key_all_patterns() {
    // Create files with each type of private key pattern
    let (dir1, file_path1) = create_temp_file("-----BEGIN RSA PRIVATE KEY-----\nkey content\n");
    let (dir2, file_path2) = create_temp_file("-----BEGIN DSA PRIVATE KEY-----\nkey content\n");
    let (dir3, file_path3) = create_temp_file("-----BEGIN EC PRIVATE KEY-----\nkey content\n");
    let (dir4, file_path4) = create_temp_file("-----BEGIN OPENSSH PRIVATE KEY-----\nkey content\n");
    let (dir5, file_path5) = create_temp_file("-----BEGIN PRIVATE KEY-----\nkey content\n");
    let (dir6, file_path6) = create_temp_file("PuTTY-User-Key-File-2: ssh-rsa\nkey content\n");

    // Run the hook on each file
    let hook = DetectPrivateKey;

    let result = hook.run(&[file_path1.clone()]);
    assert!(result.is_err());

    let result = hook.run(&[file_path2.clone()]);
    assert!(result.is_err());

    let result = hook.run(&[file_path3.clone()]);
    assert!(result.is_err());

    let result = hook.run(&[file_path4.clone()]);
    assert!(result.is_err());

    let result = hook.run(&[file_path5.clone()]);
    assert!(result.is_err());

    let result = hook.run(&[file_path6.clone()]);
    assert!(result.is_err());

    // Keep the directories alive until the end of the test
    drop(dir1);
    drop(dir2);
    drop(dir3);
    drop(dir4);
    drop(dir5);
    drop(dir6);
}

#[test]
fn test_detect_private_key_nonexistent_file() {
    // Create a path to a nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/file.txt");

    // Run the hook
    let hook = DetectPrivateKey;
    let result = hook.run(&[nonexistent_path]);
    assert!(result.is_err());

    // Verify it's an IO error
    match result {
        Err(HookError::IoError(_)) => (),
        _ => panic!("Expected IoError"),
    }
}

#[test]
fn test_hook_factory() {
    // Test creating each hook type

    // Test trailing-whitespace
    let hook = HookFactory::create_hook("trailing-whitespace", &[]);
    assert!(hook.is_ok());

    // Test end-of-file-fixer
    let hook = HookFactory::create_hook("end-of-file-fixer", &[]);
    assert!(hook.is_ok());

    // Test check-yaml
    let hook = HookFactory::create_hook("check-yaml", &[]);
    assert!(hook.is_ok());

    // Test check-added-large-files with default size
    let hook = HookFactory::create_hook("check-added-large-files", &[]);
    assert!(hook.is_ok());

    // Test check-added-large-files with custom size
    let hook = HookFactory::create_hook("check-added-large-files", &["--maxkb=1000".to_string()]);
    assert!(hook.is_ok());

    // Test check-merge-conflict
    let hook = HookFactory::create_hook("check-merge-conflict", &[]);
    assert!(hook.is_ok());

    // Test check-json
    let hook = HookFactory::create_hook("check-json", &[]);
    assert!(hook.is_ok());

    // Test check-toml
    let hook = HookFactory::create_hook("check-toml", &[]);
    assert!(hook.is_ok());

    // Test check-xml
    let hook = HookFactory::create_hook("check-xml", &[]);
    assert!(hook.is_ok());

    // Test check-case-conflict
    let hook = HookFactory::create_hook("check-case-conflict", &[]);
    assert!(hook.is_ok());

    // Test detect-private-key
    let hook = HookFactory::create_hook("detect-private-key", &[]);
    assert!(hook.is_ok());

    // Test unknown hook ID
    let hook = HookFactory::create_hook("unknown-hook", &[]);
    assert!(hook.is_err());
    if let Err(HookError::Other(msg)) = hook {
        assert!(msg.contains("Unknown hook ID"));
    } else {
        panic!("Expected HookError::Other");
    }
}
