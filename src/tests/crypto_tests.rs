use crate::crypto::*;

#[test]
fn test_lock_memory_empty_data() {
    let empty_data: &[u8] = &[];
    // Should not panic or crash
    lock_memory(empty_data);
}

#[test]
fn test_unlock_memory_empty_data() {
    let empty_data: &[u8] = &[];
    // Should not panic or crash
    unlock_memory(empty_data);
}

#[test]
fn test_secure_overwrite_empty_data() {
    let empty_data: &mut [u8] = &mut [];
    // Should not panic or crash
    secure_overwrite(empty_data);
}

#[test]
fn test_lock_unlock_memory_normal_data() {
    let data = b"sensitive data";
    // Should not panic or crash
    lock_memory(data);
    unlock_memory(data);
}

#[test]
fn test_secure_overwrite_data() {
    let mut data = [0x41u8; 16]; // Fill with 'A'

    // Verify initial state
    assert_eq!(data, [0x41u8; 16]);

    secure_overwrite(&mut data);

    // After secure overwrite, data should be zeroed
    assert_eq!(data, [0x00u8; 16]);
}

#[test]
fn test_secure_overwrite_various_sizes() {
    for size in [1, 8, 32, 64, 128, 256, 1024] {
        let mut data = vec![0xFFu8; size];
        secure_overwrite(&mut data);

        // After secure overwrite, all bytes should be zero
        assert!(data.iter().all(|&b| b == 0));
    }
}

#[test]
fn test_memory_operations_sequence() {
    let data = b"test data for memory operations";

    // Test the typical sequence: lock -> unlock
    lock_memory(data);
    unlock_memory(data);

    // Should complete without panicking
}

#[test]
fn test_secure_overwrite_preserves_length() {
    for size in [0, 1, 10, 100] {
        let mut data = vec![0xABu8; size];
        let original_len = data.len();

        secure_overwrite(&mut data);

        assert_eq!(data.len(), original_len);
    }
}
