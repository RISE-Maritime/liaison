use anyhow::{Context, Result};
use std::path::PathBuf;

#[cfg(windows)]
use windows::core::PCSTR;
#[cfg(windows)]
use windows::Win32::Foundation::MAX_PATH;
#[cfg(windows)]
use windows::Win32::System::LibraryLoader::{
    GetModuleFileNameA, GetModuleHandleExA, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
};

#[cfg(unix)]
use std::ffi::c_void;

/// Get the base directory of the FMU.
///
/// This function retrieves the path to the shared library containing this code,
/// then returns the grandparent directory (i.e., parent of parent).
///
/// On Windows, it uses GetModuleHandleEx and GetModuleFileName.
/// On Unix systems, it uses dladdr.
///
/// # Returns
///
/// The base directory path as a String.
///
/// # Errors
///
/// Returns an error if:
/// - Unable to retrieve the shared library handle or path
/// - The library path doesn't have a grandparent directory
#[cfg(windows)]
pub fn get_base_directory() -> Result<String> {
    use std::mem::MaybeUninit;
    use windows::Win32::Foundation::HMODULE;

    unsafe {
        let mut h_module = MaybeUninit::<HMODULE>::uninit();

        // Get the handle for the DLL containing this function
        let result = GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCSTR(get_base_directory as *const u8),
            h_module.as_mut_ptr(),
        );

        if result.is_err() {
            anyhow::bail!("Failed to retrieve the shared library handle");
        }

        let h_module = h_module.assume_init();

        // Retrieve the full path of the DLL
        let mut path_buf = vec![0u8; MAX_PATH as usize];
        let len = GetModuleFileNameA(h_module, &mut path_buf);

        if len == 0 {
            anyhow::bail!("Failed to retrieve shared library path");
        }

        // Convert to String, trimming null bytes
        let path_str = std::str::from_utf8(&path_buf[..len as usize])
            .context("Invalid UTF-8 in library path")?;

        // Get the grandparent directory
        let library_path = PathBuf::from(path_str);
        let base_dir = library_path
            .parent()
            .and_then(|p| p.parent())
            .context("Unable to get grandparent directory of library path")?;

        Ok(base_dir.to_string_lossy().to_string())
    }
}

#[cfg(unix)]
#[repr(C)]
struct DlInfo {
    dli_fname: *const libc::c_char,
    dli_fbase: *mut c_void,
    dli_sname: *const libc::c_char,
    dli_saddr: *mut c_void,
}

#[cfg(unix)]
extern "C" {
    fn dladdr(addr: *const c_void, info: *mut DlInfo) -> libc::c_int;
}

#[cfg(unix)]
pub fn get_base_directory() -> Result<String> {
    use std::ffi::CStr;

    unsafe {
        let mut dl_info: DlInfo = std::mem::zeroed();

        // Retrieve the shared library path
        let result = dladdr(get_base_directory as *const c_void, &mut dl_info);

        if result == 0 {
            anyhow::bail!("Failed to retrieve shared library path");
        }

        if dl_info.dli_fname.is_null() {
            anyhow::bail!("Shared library path is null");
        }

        // Convert C string to Rust string
        let c_str = CStr::from_ptr(dl_info.dli_fname);
        let path_str = c_str.to_str().context("Invalid UTF-8 in library path")?;

        // Get the grandparent directory
        let library_path = PathBuf::from(path_str);
        let base_dir = library_path
            .parent()
            .and_then(|p| p.parent())
            .context("Unable to get grandparent directory of library path")?;

        Ok(base_dir.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_base_directory_returns_ok() {
        // This test verifies that the function runs without panicking
        // The actual path will vary depending on where the test is run
        let result = get_base_directory();
        assert!(result.is_ok(), "get_base_directory should succeed");

        let path = result.unwrap();
        assert!(!path.is_empty(), "Base directory should not be empty");
    }

    #[test]
    fn test_get_base_directory_returns_valid_path() {
        let result = get_base_directory();
        assert!(result.is_ok());

        let path_str = result.unwrap();
        let path = Path::new(&path_str);

        // The path should be a valid filesystem path
        // We can't guarantee it exists in all test environments, but it should be well-formed
        assert!(path.is_absolute() || !path_str.is_empty());
    }

    #[test]
    fn test_get_base_directory_consistency() {
        // Calling the function multiple times should return the same result
        let result1 = get_base_directory();
        let result2 = get_base_directory();

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_get_base_directory_contains_grandparent() {
        // The function should return the grandparent directory of the library
        let result = get_base_directory();
        assert!(result.is_ok());

        let path_str = result.unwrap();
        let path = Path::new(&path_str);

        // The path should have at least one component (not be empty)
        let path_components: Vec<_> = path.components().collect();
        assert!(
            !path_components.is_empty(),
            "Path should have at least one component"
        );

        // The path should be a valid directory path
        assert!(!path.as_os_str().is_empty(), "Path should not be empty");
    }

    #[test]
    fn test_get_base_directory_is_directory_like() {
        let result = get_base_directory();
        assert!(result.is_ok());

        let path_str = result.unwrap();

        // Should not end with file extensions
        assert!(!path_str.ends_with(".dll"));
        assert!(!path_str.ends_with(".so"));
        assert!(!path_str.ends_with(".dylib"));
        assert!(!path_str.ends_with(".exe"));
    }

    #[test]
    fn test_get_base_directory_no_trailing_separator() {
        let result = get_base_directory();
        assert!(result.is_ok());

        let path_str = result.unwrap();

        // Path should not have trailing separator (on most systems)
        // This is a quality check for consistent path handling
        #[cfg(windows)]
        assert!(!path_str.ends_with('\\') || path_str.len() <= 3); // Allow for "C:\"

        #[cfg(unix)]
        assert!(!path_str.ends_with('/') || path_str == "/");
    }
}
