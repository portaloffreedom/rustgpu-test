use spirv_builder::MetadataPrintout;
use spirv_builder::{SpirvBuilder, SpirvBuilderError};

fn main() -> Result<(), SpirvBuilderError> {
    let path_to_shader = "../shaders";
    let target = "spirv-unknown-spv1.5";
    let _compile_result = SpirvBuilder::new(path_to_shader, target)
        .multimodule(false)
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    println!("cargo:rerun-if-changed={}/Cargo.toml", path_to_shader);

    Ok(())
}
