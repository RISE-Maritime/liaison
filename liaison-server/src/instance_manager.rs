//! Instance Manager for FMU Instances
//!
//! This module provides thread-safe management of FMU instances.
//! It maintains a mapping between instance indices and FMI instance pointers,
//! allowing multiple instances to be tracked and accessed safely.
//!
//! # Thread Safety
//!
//! The `InstanceManager` uses `Arc<Mutex<...>>` to provide thread-safe access
//! to the instance map. This allows the manager to be safely shared across
//! multiple threads handling concurrent FMU operations.
//!
//! # Example
//!
//! ```ignore
//! use instance_manager::InstanceManager;
//! use std::ffi::c_void;
//!
//! let manager = InstanceManager::new();
//! let instance_ptr = /* create FMU instance */;
//! let index = manager.add_instance(instance_ptr);
//! let retrieved = manager.get_instance(index)?;
//! manager.remove_instance(index)?;
//! ```

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

/// Error types for instance manager operations
#[derive(Debug, Clone, PartialEq)]
pub enum InstanceError {
    /// Instance with the given index was not found
    InstanceNotFound(i32),
    /// Invalid instance index provided (e.g., negative)
    InvalidIndex(i32),
    /// Lock was poisoned due to panic in another thread
    LockPoisoned,
}

impl std::fmt::Display for InstanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceError::InstanceNotFound(index) => {
                write!(f, "Instance with index {} not found", index)
            }
            InstanceError::InvalidIndex(index) => {
                write!(f, "Invalid instance index: {}", index)
            }
            InstanceError::LockPoisoned => {
                write!(f, "Instance manager lock was poisoned")
            }
        }
    }
}

impl std::error::Error for InstanceError {}

/// Result type for instance manager operations
pub type Result<T> = std::result::Result<T, InstanceError>;

/// Internal state of the instance manager
struct InstanceManagerState {
    /// Map of instance indices to FMI instance pointers
    instances: HashMap<i32, *mut c_void>,
    /// Next available instance index
    next_index: i32,
}

/// Thread-safe manager for FMU instances
///
/// The `InstanceManager` maintains a mapping between instance indices and
/// FMI instance pointers. It is designed to be shared across threads using
/// `Arc` and provides interior mutability through `Mutex`.
///
/// # Thread Safety
///
/// This struct is `Send` and `Sync` safe and can be shared across threads.
/// All operations acquire a lock on the internal state.
///
/// # Implementation Notes
///
/// Based on the C++ implementation in liaison.cpp:
/// - Lines 176-177: Global instances map and nextIndex counter
/// - Lines 217-223: getInstance function with error handling
/// - Lines 358-380: Instance creation and index assignment
/// - Lines 475-495: Instance removal and cleanup
#[derive(Clone)]
pub struct InstanceManager {
    state: Arc<Mutex<InstanceManagerState>>,
}

impl InstanceManager {
    /// Create a new instance manager
    ///
    /// Initializes an empty instance map with the next index set to 0.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let manager = InstanceManager::new();
    /// ```
    pub fn new() -> Self {
        InstanceManager {
            state: Arc::new(Mutex::new(InstanceManagerState {
                instances: HashMap::new(),
                next_index: 0,
            })),
        }
    }

    /// Add a new FMU instance to the manager
    ///
    /// Stores the instance pointer and returns a unique index that can be used
    /// to retrieve or remove the instance later.
    ///
    /// # Arguments
    ///
    /// * `instance` - Raw pointer to the FMI instance (fmi3Instance)
    ///
    /// # Returns
    ///
    /// The assigned instance index
    ///
    /// # Panics
    ///
    /// Panics if the lock is poisoned (another thread panicked while holding the lock)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let instance_ptr = fmi3_instantiate_co_simulation(...);
    /// let index = manager.add_instance(instance_ptr);
    /// println!("Created instance with index: {}", index);
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// Based on liaison.cpp lines 376-379:
    /// ```cpp
    /// instances[nextIndex] = instance;
    /// output.set_instance_index(nextIndex);
    /// nextIndex++;
    /// ```
    pub fn add_instance(&self, instance: *mut c_void) -> i32 {
        let mut state = self.state.lock().unwrap_or_else(|e| {
            // If the lock is poisoned, we still want to access the data
            // This matches the behavior of the C++ version which has no lock
            e.into_inner()
        });

        let index = state.next_index;
        state.instances.insert(index, instance);
        state.next_index += 1;

        index
    }

    /// Retrieve an FMU instance by its index
    ///
    /// Returns the instance pointer associated with the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The instance index returned from `add_instance`
    ///
    /// # Returns
    ///
    /// * `Ok(*mut c_void)` - The instance pointer if found
    /// * `Err(InstanceError::InvalidIndex)` - If the index is negative
    /// * `Err(InstanceError::InstanceNotFound)` - If no instance exists at that index
    ///
    /// # Examples
    ///
    /// ```ignore
    /// match manager.get_instance(index) {
    ///     Ok(instance_ptr) => {
    ///         // Use the instance
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Error: {}", e);
    ///     }
    /// }
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// Based on liaison.cpp lines 217-223:
    /// ```cpp
    /// fmi3Instance getInstance(int index) {
    ///     auto it = instances.find(index);
    ///     if (it == instances.end()) {
    ///         throw std::out_of_range("Instance index out of range.");
    ///     }
    ///     return it->second;
    /// }
    /// ```
    pub fn get_instance(&self, index: i32) -> Result<*mut c_void> {
        // Validate index
        if index < 0 {
            return Err(InstanceError::InvalidIndex(index));
        }

        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        state
            .instances
            .get(&index)
            .copied()
            .ok_or(InstanceError::InstanceNotFound(index))
    }

    /// Remove an FMU instance from the manager
    ///
    /// Removes the instance from the internal map. Note that this does not
    /// free the instance itself - the caller is responsible for calling
    /// `fmi3FreeInstance` on the pointer before or after removal.
    ///
    /// # Arguments
    ///
    /// * `index` - The instance index to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instance was successfully removed
    /// * `Err(InstanceError::InvalidIndex)` - If the index is negative
    /// * `Err(InstanceError::InstanceNotFound)` - If no instance exists at that index
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // First free the FMU instance
    /// let instance_ptr = manager.get_instance(index)?;
    /// fmi3_free_instance(instance_ptr);
    ///
    /// // Then remove from manager
    /// manager.remove_instance(index)?;
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// Based on liaison.cpp lines 486-491:
    /// ```cpp
    /// try {
    ///     auto it = instances.find(input.instance_index());
    ///     instances.erase(it);
    /// } catch (std::runtime_error& error) {
    ///     spdlog::error("Failed to erase instance from instances.");
    /// }
    /// ```
    pub fn remove_instance(&self, index: i32) -> Result<()> {
        // Validate index
        if index < 0 {
            return Err(InstanceError::InvalidIndex(index));
        }

        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        state
            .instances
            .remove(&index)
            .map(|_| ())
            .ok_or(InstanceError::InstanceNotFound(index))
    }

    /// Get the number of active instances
    ///
    /// Returns the count of currently managed instances.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let count = manager.instance_count();
    /// println!("Managing {} instances", count);
    /// ```
    pub fn instance_count(&self) -> usize {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.instances.len()
    }

    /// Check if an instance exists at the given index
    ///
    /// # Arguments
    ///
    /// * `index` - The instance index to check
    ///
    /// # Returns
    ///
    /// `true` if an instance exists at the index, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if manager.contains_instance(index) {
    ///     println!("Instance exists");
    /// }
    /// ```
    pub fn contains_instance(&self, index: i32) -> bool {
        if index < 0 {
            return false;
        }

        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.instances.contains_key(&index)
    }
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: InstanceManager can be safely sent between threads because
// all access to the shared state is protected by a Mutex
unsafe impl Send for InstanceManager {}

// Safety: InstanceManager can be safely shared between threads because
// all access to the shared state is protected by a Mutex
unsafe impl Sync for InstanceManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_instance_manager() {
        let manager = InstanceManager::new();
        assert_eq!(manager.instance_count(), 0);
    }

    #[test]
    fn test_add_instance() {
        let manager = InstanceManager::new();
        let dummy_ptr = 0x1234 as *mut c_void;

        let index1 = manager.add_instance(dummy_ptr);
        assert_eq!(index1, 0);

        let index2 = manager.add_instance(dummy_ptr);
        assert_eq!(index2, 1);

        assert_eq!(manager.instance_count(), 2);
    }

    #[test]
    fn test_get_instance() {
        let manager = InstanceManager::new();
        let dummy_ptr = 0x1234 as *mut c_void;

        let index = manager.add_instance(dummy_ptr);
        let retrieved = manager.get_instance(index).unwrap();

        assert_eq!(retrieved, dummy_ptr);
    }

    #[test]
    fn test_get_nonexistent_instance() {
        let manager = InstanceManager::new();
        let result = manager.get_instance(999);

        assert_eq!(result, Err(InstanceError::InstanceNotFound(999)));
    }

    #[test]
    fn test_get_invalid_index() {
        let manager = InstanceManager::new();
        let result = manager.get_instance(-1);

        assert_eq!(result, Err(InstanceError::InvalidIndex(-1)));
    }

    #[test]
    fn test_remove_instance() {
        let manager = InstanceManager::new();
        let dummy_ptr = 0x1234 as *mut c_void;

        let index = manager.add_instance(dummy_ptr);
        assert_eq!(manager.instance_count(), 1);

        let result = manager.remove_instance(index);
        assert!(result.is_ok());
        assert_eq!(manager.instance_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_instance() {
        let manager = InstanceManager::new();
        let result = manager.remove_instance(999);

        assert_eq!(result, Err(InstanceError::InstanceNotFound(999)));
    }

    #[test]
    fn test_remove_invalid_index() {
        let manager = InstanceManager::new();
        let result = manager.remove_instance(-1);

        assert_eq!(result, Err(InstanceError::InvalidIndex(-1)));
    }

    #[test]
    fn test_contains_instance() {
        let manager = InstanceManager::new();
        let dummy_ptr = 0x1234 as *mut c_void;

        let index = manager.add_instance(dummy_ptr);
        assert!(manager.contains_instance(index));
        assert!(!manager.contains_instance(999));
        assert!(!manager.contains_instance(-1));
    }

    #[test]
    fn test_multiple_instances() {
        let manager = InstanceManager::new();
        let ptr1 = 0x1000 as *mut c_void;
        let ptr2 = 0x2000 as *mut c_void;
        let ptr3 = 0x3000 as *mut c_void;

        let idx1 = manager.add_instance(ptr1);
        let idx2 = manager.add_instance(ptr2);
        let idx3 = manager.add_instance(ptr3);

        assert_eq!(manager.get_instance(idx1).unwrap(), ptr1);
        assert_eq!(manager.get_instance(idx2).unwrap(), ptr2);
        assert_eq!(manager.get_instance(idx3).unwrap(), ptr3);

        manager.remove_instance(idx2).unwrap();
        assert_eq!(manager.instance_count(), 2);
        assert!(manager.get_instance(idx2).is_err());
        assert_eq!(manager.get_instance(idx1).unwrap(), ptr1);
        assert_eq!(manager.get_instance(idx3).unwrap(), ptr3);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let manager = InstanceManager::new();
        let manager_clone1 = manager.clone();
        let manager_clone2 = manager.clone();

        let handle1 = thread::spawn(move || {
            for i in 0..100 {
                let ptr = (i * 1000) as *mut c_void;
                manager_clone1.add_instance(ptr);
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 0..100 {
                let ptr = (i * 2000) as *mut c_void;
                manager_clone2.add_instance(ptr);
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert_eq!(manager.instance_count(), 200);
    }
}
