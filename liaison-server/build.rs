use std::io::Result;

fn main() -> Result<()> {
    // Generate Rust code from protobuf definitions
    prost_build::compile_protos(&["../src/fmi3.proto"], &["../src/"])?;
    Ok(())
}
