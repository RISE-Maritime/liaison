// Liaison FMI Client Library
// This library implements FMI 3.0 functions that communicate with a remote FMU server via Zenoh

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod conversions;
pub mod fmi3;
mod placeholder;
pub mod proto;
mod utils;

// Re-export the FMI functions
pub use fmi3::*;
