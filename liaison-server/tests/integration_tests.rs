// Integration tests for liaison-server
//
// These tests cover:
// - InstanceManager operations
// - FMU loader functionality
// - Utility functions (file operations, directory creation)
// - FMU creator functionality
// - Callback functions
//
// Note: Some tests require actual FMU files or mock dynamic libraries
// and are marked as #[ignore] by default. They can be run with:
// cargo test -- --ignored

use liaison_server::{
    callbacks::{fmi3_log_message, status_to_proto, CallbackContext},
    fmu_loader::{construct_library_path, fmi3Status},
    instance_manager::{InstanceError, InstanceManager},
    proto::Status as ProtoStatus,
    utils::{add_file_to_fmu, create_directories, unzip_fmu},
};
use std::ffi::{c_void, CString};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

// ============================================================================
// InstanceManager Tests
// ============================================================================

#[test]
fn test_instance_manager_lifecycle() {
    let manager = InstanceManager::new();

    // Initially empty
    assert_eq!(manager.instance_count(), 0);

    // Add instances
    let ptr1 = 0x1000 as *mut c_void;
    let ptr2 = 0x2000 as *mut c_void;
    let ptr3 = 0x3000 as *mut c_void;

    let idx1 = manager.add_instance(ptr1);
    let idx2 = manager.add_instance(ptr2);
    let idx3 = manager.add_instance(ptr3);

    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(idx3, 2);
    assert_eq!(manager.instance_count(), 3);

    // Retrieve instances
    assert_eq!(manager.get_instance(idx1).unwrap(), ptr1);
    assert_eq!(manager.get_instance(idx2).unwrap(), ptr2);
    assert_eq!(manager.get_instance(idx3).unwrap(), ptr3);

    // Check contains
    assert!(manager.contains_instance(idx1));
    assert!(manager.contains_instance(idx2));
    assert!(manager.contains_instance(idx3));
    assert!(!manager.contains_instance(999));

    // Remove middle instance
    assert!(manager.remove_instance(idx2).is_ok());
    assert_eq!(manager.instance_count(), 2);
    assert!(!manager.contains_instance(idx2));

    // Verify idx1 and idx3 still exist
    assert_eq!(manager.get_instance(idx1).unwrap(), ptr1);
    assert_eq!(manager.get_instance(idx3).unwrap(), ptr3);

    // Remove remaining instances
    assert!(manager.remove_instance(idx1).is_ok());
    assert!(manager.remove_instance(idx3).is_ok());
    assert_eq!(manager.instance_count(), 0);
}

#[test]
fn test_instance_manager_error_handling() {
    let manager = InstanceManager::new();

    // Test invalid index (negative)
    let result = manager.get_instance(-1);
    assert_eq!(result, Err(InstanceError::InvalidIndex(-1)));

    let result = manager.remove_instance(-1);
    assert_eq!(result, Err(InstanceError::InvalidIndex(-1)));

    // Test non-existent instance
    let result = manager.get_instance(999);
    assert_eq!(result, Err(InstanceError::InstanceNotFound(999)));

    let result = manager.remove_instance(999);
    assert_eq!(result, Err(InstanceError::InstanceNotFound(999)));
}

#[test]
fn test_instance_manager_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(InstanceManager::new());
    let mut handles = vec![];

    // Spawn multiple threads that add instances
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                let ptr = ((i * 100) + j) as *mut c_void;
                manager_clone.add_instance(ptr);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all 100 instances were added
    assert_eq!(manager.instance_count(), 100);
}

#[test]
fn test_instance_manager_default() {
    let manager = InstanceManager::default();
    assert_eq!(manager.instance_count(), 0);
}

#[test]
fn test_instance_manager_multiple_operations() {
    let manager = InstanceManager::new();

    // Add multiple instances
    let mut indices = vec![];
    for i in 0..50 {
        let ptr = (i * 1000) as *mut c_void;
        let idx = manager.add_instance(ptr);
        indices.push(idx);
    }

    assert_eq!(manager.instance_count(), 50);

    // Remove every other instance
    for i in (0..50).step_by(2) {
        assert!(manager.remove_instance(indices[i]).is_ok());
    }

    assert_eq!(manager.instance_count(), 25);

    // Verify remaining instances are still accessible
    for i in (1..50).step_by(2) {
        let ptr = (i * 1000) as *mut c_void;
        assert_eq!(manager.get_instance(indices[i]).unwrap(), ptr);
    }
}

// ============================================================================
// FMU Loader Tests
// ============================================================================

#[test]
fn test_construct_library_path_linux() {
    #[cfg(target_os = "linux")]
    {
        let path = construct_library_path("/tmp/fmu", "MyModel");
        assert_eq!(path, "/tmp/fmu/binaries/x86_64-linux/MyModel.so");

        let path = construct_library_path("/home/user/models/extracted", "BouncingBall");
        assert_eq!(
            path,
            "/home/user/models/extracted/binaries/x86_64-linux/BouncingBall.so"
        );
    }
}

#[test]
fn test_construct_library_path_windows() {
    #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
    {
        let path = construct_library_path("C:\\temp\\fmu", "MyModel");
        assert_eq!(
            path,
            "C:\\temp\\fmu/binaries/x86_64-windows/MyModel.dll"
        );
    }

    #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
    {
        let path = construct_library_path("C:\\temp\\fmu", "MyModel");
        assert_eq!(path, "C:\\temp\\fmu/binaries/x86-windows/MyModel.dll");
    }
}

#[test]
fn test_construct_library_path_special_characters() {
    let path = construct_library_path("/tmp/my-fmu_123/test", "Model-v2.0_beta");

    #[cfg(target_os = "linux")]
    assert_eq!(
        path,
        "/tmp/my-fmu_123/test/binaries/x86_64-linux/Model-v2.0_beta.so"
    );

    #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
    assert_eq!(
        path,
        "/tmp/my-fmu_123/test/binaries/x86_64-windows/Model-v2.0_beta.dll"
    );
}

// Note: Testing FmuLibrary::new requires actual FMU binaries, which should be
// tested with system/integration tests using real FMU files

// ============================================================================
// Utility Function Tests
// ============================================================================

#[test]
fn test_create_directories() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("test/nested/directories");

    // Directory should not exist initially
    assert!(!test_path.exists());

    // Create the directories
    create_directories(&test_path).unwrap();

    // Directory should now exist
    assert!(test_path.exists());
    assert!(test_path.is_dir());
}

#[test]
fn test_create_directories_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("existing");

    // Create directory first time
    create_directories(&test_path).unwrap();
    assert!(test_path.exists());

    // Creating again should not fail
    create_directories(&test_path).unwrap();
    assert!(test_path.exists());
}

#[test]
fn test_unzip_fmu() {
    let temp_dir = TempDir::new().unwrap();
    let fmu_path = temp_dir.path().join("test.fmu");

    // Create a simple ZIP file (FMU is just a ZIP)
    let file = File::create(&fmu_path).unwrap();
    let mut zip = ZipWriter::new(file);

    // Add modelDescription.xml
    zip.start_file("modelDescription.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"<?xml version=\"1.0\"?><fmiModelDescription/>")
        .unwrap();

    // Add a binary directory structure
    zip.start_file(
        "binaries/x86_64-linux/model.so",
        SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(b"fake binary content").unwrap();

    zip.finish().unwrap();

    // Unzip the FMU
    let extracted_path = unzip_fmu(&fmu_path).unwrap();

    // Verify files were extracted
    let extracted_dir = PathBuf::from(&extracted_path);
    assert!(extracted_dir.join("modelDescription.xml").exists());
    assert!(extracted_dir
        .join("binaries/x86_64-linux/model.so")
        .exists());

    // Verify content
    let content = fs::read_to_string(extracted_dir.join("modelDescription.xml")).unwrap();
    assert!(content.contains("fmiModelDescription"));
}

#[test]
fn test_add_file_to_fmu() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.fmu");

    // Create a test file to add
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test content").unwrap();

    // Create ZIP and add file
    let file = File::create(&output_path).unwrap();
    let mut zip = ZipWriter::new(file);

    add_file_to_fmu(&mut zip, &test_file, "archived/test.txt").unwrap();

    zip.finish().unwrap();

    // Extract and verify
    let extracted_path = unzip_fmu(&output_path).unwrap();
    let extracted_file = PathBuf::from(extracted_path).join("archived/test.txt");
    assert!(extracted_file.exists());

    let content = fs::read_to_string(extracted_file).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_add_file_to_fmu_binary_content() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("binary.fmu");

    // Create a binary file
    let test_file = temp_dir.path().join("binary.bin");
    let binary_data: Vec<u8> = (0..255).collect();
    fs::write(&test_file, &binary_data).unwrap();

    // Create ZIP and add file
    let file = File::create(&output_path).unwrap();
    let mut zip = ZipWriter::new(file);

    add_file_to_fmu(&mut zip, &test_file, "data/binary.bin").unwrap();

    zip.finish().unwrap();

    // Extract and verify
    let extracted_path = unzip_fmu(&output_path).unwrap();
    let extracted_file = PathBuf::from(extracted_path).join("data/binary.bin");
    assert!(extracted_file.exists());

    let content = fs::read(&extracted_file).unwrap();
    assert_eq!(content, binary_data);
}

#[test]
fn test_add_file_to_fmu_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.fmu");

    let file = File::create(&output_path).unwrap();
    let mut zip = ZipWriter::new(file);

    let nonexistent = temp_dir.path().join("does_not_exist.txt");
    let result = add_file_to_fmu(&mut zip, &nonexistent, "test.txt");

    assert!(result.is_err());
}

// ============================================================================
// FMU Creator Tests
// ============================================================================

// Note: Full FMU creation tests are marked #[ignore] because they require:
// - Actual FMU files
// - Liaison FMI client library binaries in ./binaries/
// - Valid file system setup
//
// Run with: cargo test -- --ignored

#[test]
#[ignore]
fn test_make_fmu_basic() {
    // This test requires:
    // 1. A valid FMU file at the specified path
    // 2. Liaison binaries in ./binaries/x86_64-linux/ and ./binaries/x86_64-windows/
    // 3. Write permissions in current directory

    use liaison_server::fmu_creator::make_fmu;

    let temp_dir = TempDir::new().unwrap();

    // Create a minimal test FMU
    let source_fmu = temp_dir.path().join("Source.fmu");
    let file = File::create(&source_fmu).unwrap();
    let mut zip = ZipWriter::new(file);

    // Add minimal modelDescription.xml
    zip.start_file("modelDescription.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<fmiModelDescription
    fmiVersion="3.0"
    modelName="TestModel"
    instantiationToken="{12345678-1234-1234-1234-123456789012}">
</fmiModelDescription>"#,
    )
    .unwrap();

    zip.finish().unwrap();

    // Attempt to create Liaison FMU (will fail without binaries)
    let result = make_fmu(source_fmu, "test-responder".to_string(), None);

    // Expected to fail due to missing binaries directory
    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_make_fmu_with_zenoh_config() {
    use liaison_server::fmu_creator::make_fmu;
    use serde_json::json;

    let temp_dir = TempDir::new().unwrap();

    // Create a minimal test FMU
    let source_fmu = temp_dir.path().join("Source.fmu");
    let file = File::create(&source_fmu).unwrap();
    let mut zip = ZipWriter::new(file);

    zip.start_file("modelDescription.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"<?xml version=\"1.0\"?><fmiModelDescription/>")
        .unwrap();

    zip.finish().unwrap();

    // Create a Zenoh config file
    let zenoh_config_path = temp_dir.path().join("zenoh.json");
    let config = json!({
        "mode": "peer",
        "connect": {
            "endpoints": ["tcp/localhost:7447"]
        }
    });
    fs::write(&zenoh_config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    // Attempt to create Liaison FMU
    let result = make_fmu(
        source_fmu,
        "test-responder".to_string(),
        Some(zenoh_config_path),
    );

    // Expected to fail due to missing binaries directory
    assert!(result.is_err());
}

// ============================================================================
// Callback Function Tests
// ============================================================================

#[test]
fn test_callback_context_creation() {
    let ctx = CallbackContext::new();
    // Verify it can be created without errors
    let _ = ctx;
}

#[test]
fn test_callback_context_default() {
    let ctx = CallbackContext::default();
    let _ = ctx;
}

#[test]
fn test_status_to_proto_conversion() {
    assert_eq!(status_to_proto(fmi3Status::fmi3OK), ProtoStatus::Ok);
    assert_eq!(
        status_to_proto(fmi3Status::fmi3Warning),
        ProtoStatus::Warning
    );
    assert_eq!(
        status_to_proto(fmi3Status::fmi3Discard),
        ProtoStatus::Discard
    );
    assert_eq!(status_to_proto(fmi3Status::fmi3Error), ProtoStatus::Error);
    assert_eq!(status_to_proto(fmi3Status::fmi3Fatal), ProtoStatus::Fatal);
}

#[test]
fn test_fmi3_log_message_callback_with_valid_strings() {
    let category = CString::new("test_category").unwrap();
    let message = CString::new("This is a test message").unwrap();

    // Test with null context (safe since we don't use it yet)
    fmi3_log_message(
        std::ptr::null_mut(),
        fmi3Status::fmi3OK,
        category.as_ptr(),
        message.as_ptr(),
    );

    // Test should complete without panicking
}

#[test]
fn test_fmi3_log_message_callback_with_null_strings() {
    // Should handle null strings gracefully
    fmi3_log_message(
        std::ptr::null_mut(),
        fmi3Status::fmi3Warning,
        std::ptr::null(),
        std::ptr::null(),
    );

    // Test should complete without panicking
}

#[test]
fn test_fmi3_log_message_all_status_levels() {
    let category = CString::new("status_test").unwrap();
    let message = CString::new("Testing status level").unwrap();

    let status_levels = vec![
        fmi3Status::fmi3OK,
        fmi3Status::fmi3Warning,
        fmi3Status::fmi3Discard,
        fmi3Status::fmi3Error,
        fmi3Status::fmi3Fatal,
    ];

    for status in status_levels {
        fmi3_log_message(
            std::ptr::null_mut(),
            status,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    // All status levels should be handled without panicking
}

#[test]
fn test_fmi3_log_message_with_special_characters() {
    let category = CString::new("special_chars").unwrap();
    let message = CString::new("Message with émojis 🚀 and ñ special çhars").unwrap();

    fmi3_log_message(
        std::ptr::null_mut(),
        fmi3Status::fmi3OK,
        category.as_ptr(),
        message.as_ptr(),
    );

    // Should handle UTF-8 properly
}

#[test]
fn test_fmi3_log_message_with_empty_strings() {
    let category = CString::new("").unwrap();
    let message = CString::new("").unwrap();

    fmi3_log_message(
        std::ptr::null_mut(),
        fmi3Status::fmi3OK,
        category.as_ptr(),
        message.as_ptr(),
    );

    // Empty strings should be handled
}

// ============================================================================
// Integration Tests Combining Multiple Components
// ============================================================================

#[test]
fn test_instance_manager_with_multiple_fmu_scenarios() {
    let manager = InstanceManager::new();

    // Simulate loading multiple FMUs
    let fmu1_instance = 0x10000 as *mut c_void;
    let fmu2_instance = 0x20000 as *mut c_void;
    let fmu3_instance = 0x30000 as *mut c_void;

    // Add instances
    let idx1 = manager.add_instance(fmu1_instance);
    let idx2 = manager.add_instance(fmu2_instance);
    let idx3 = manager.add_instance(fmu3_instance);

    // Simulate FMU operations
    for _ in 0..100 {
        // Get instances multiple times (simulating doStep operations)
        assert!(manager.get_instance(idx1).is_ok());
        assert!(manager.get_instance(idx2).is_ok());
        assert!(manager.get_instance(idx3).is_ok());
    }

    // Simulate terminating one FMU
    manager.remove_instance(idx2).unwrap();
    assert_eq!(manager.instance_count(), 2);

    // Other instances should still work
    assert!(manager.get_instance(idx1).is_ok());
    assert!(manager.get_instance(idx3).is_ok());
    assert!(manager.get_instance(idx2).is_err());

    // Clean up
    manager.remove_instance(idx1).unwrap();
    manager.remove_instance(idx3).unwrap();
    assert_eq!(manager.instance_count(), 0);
}

#[test]
fn test_file_operations_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested directory structure
    let nested_path = temp_dir.path().join("fmu/binaries/x86_64-linux");
    create_directories(&nested_path).unwrap();
    assert!(nested_path.exists());

    // Create a test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"test data").unwrap();

    // Create an FMU-like ZIP
    let fmu_path = temp_dir.path().join("test.fmu");
    let file = File::create(&fmu_path).unwrap();
    let mut zip = ZipWriter::new(file);

    // Add multiple files
    add_file_to_fmu(&mut zip, &test_file, "file1.txt").unwrap();
    add_file_to_fmu(&mut zip, &test_file, "nested/file2.txt").unwrap();

    zip.finish().unwrap();

    // Unzip and verify
    let extracted = unzip_fmu(&fmu_path).unwrap();
    let extracted_dir = PathBuf::from(extracted);

    assert!(extracted_dir.join("file1.txt").exists());
    assert!(extracted_dir.join("nested/file2.txt").exists());
}

#[test]
fn test_error_handling_robustness() {
    let manager = InstanceManager::new();

    // Test multiple error conditions
    let error_cases = vec![-1, -100, -999];

    for idx in error_cases {
        assert!(matches!(
            manager.get_instance(idx),
            Err(InstanceError::InvalidIndex(_))
        ));
        assert!(matches!(
            manager.remove_instance(idx),
            Err(InstanceError::InvalidIndex(_))
        ));
    }

    // Test non-existent instances
    let nonexistent_cases = vec![0, 1, 100, 999, 9999];

    for idx in nonexistent_cases {
        assert!(matches!(
            manager.get_instance(idx),
            Err(InstanceError::InstanceNotFound(_))
        ));
        assert!(matches!(
            manager.remove_instance(idx),
            Err(InstanceError::InstanceNotFound(_))
        ));
    }
}

// ============================================================================
// Performance and Stress Tests
// ============================================================================

#[test]
fn test_instance_manager_many_instances() {
    let manager = InstanceManager::new();

    // Add many instances
    let count = 1000;
    let mut indices = Vec::with_capacity(count);

    for i in 0..count {
        let ptr = (i * 0x1000) as *mut c_void;
        let idx = manager.add_instance(ptr);
        indices.push((idx, ptr));
    }

    assert_eq!(manager.instance_count(), count);

    // Verify all can be retrieved
    for (idx, ptr) in &indices {
        assert_eq!(manager.get_instance(*idx).unwrap(), *ptr);
    }

    // Remove all
    for (idx, _) in indices {
        manager.remove_instance(idx).unwrap();
    }

    assert_eq!(manager.instance_count(), 0);
}

#[test]
fn test_concurrent_instance_operations() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(InstanceManager::new());
    let mut handles = vec![];

    // Create instances from multiple threads
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        handles.push(thread::spawn(move || {
            let mut local_indices = vec![];
            for j in 0..20 {
                let ptr = ((i * 1000) + j) as *mut c_void;
                let idx = manager_clone.add_instance(ptr);
                local_indices.push(idx);
            }
            local_indices
        }));
    }

    // Collect all indices
    let mut all_indices = vec![];
    for handle in handles {
        all_indices.extend(handle.join().unwrap());
    }

    assert_eq!(manager.instance_count(), 100);

    // Verify all instances from multiple threads
    let mut verify_handles = vec![];
    for chunk in all_indices.chunks(20) {
        let manager_clone = Arc::clone(&manager);
        let chunk_vec = chunk.to_vec();
        verify_handles.push(thread::spawn(move || {
            for idx in chunk_vec {
                assert!(manager_clone.get_instance(idx).is_ok());
            }
        }));
    }

    for handle in verify_handles {
        handle.join().unwrap();
    }
}
