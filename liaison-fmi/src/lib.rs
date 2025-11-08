// Liaison FMI Client Library
// This library implements FMI 3.0 functions that communicate with a remote FMU server via Zenoh

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod fmi3;
mod proto;
mod placeholder;

// Re-export the FMI functions
pub use fmi3::*;
