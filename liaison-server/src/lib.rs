// Library exports for liaison-server
// Exposes modules for integration testing and library usage

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

pub mod callbacks;
pub mod fmu_creator;
pub mod fmu_loader;
pub mod instance_manager;
pub mod queryable_handlers;
pub mod server;
pub mod utils;
