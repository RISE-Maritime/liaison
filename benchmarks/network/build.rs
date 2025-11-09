use std::io::Result;

fn main() -> Result<()> {
    // Compile the FMI protocol buffer definitions
    // This assumes the proto file is in the workspace root src directory
    prost_build::compile_protos(&["../../src/fmi3.proto"], &["../../src/"])?;
    Ok(())
}
