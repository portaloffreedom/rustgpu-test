use spirv_builder::{MetadataPrintout, SpirvMetadata};
use spirv_builder::{SpirvBuilder, SpirvBuilderError};

fn main() -> Result<(), SpirvBuilderError> {
    let target = "spirv-unknown-spv1.5";
    let path_to_shaders = [
        "../shaders/simple_compute",
        "../shaders/fractal",
        "../shaders/simple_graphics",
    ];

    for path_to_shader in path_to_shaders {
        let cargo_file = format!("{}/Cargo.toml", path_to_shader);
        println!("cargo:rerun-if-changed={}", cargo_file);

        // let _compile_result = SpirvBuilder::new(path_to_shader, target)
        //     .multimodule(false)
        //     .print_metadata(MetadataPrintout::Full)
        //     .build()?;

        let compile_result = SpirvBuilder::new(path_to_shader, target)
            .multimodule(true)
            .spirv_metadata(SpirvMetadata::NameVariables)
            .print_metadata(MetadataPrintout::DependencyOnly)
            .build()?;

        let shader_name = cargo_toml::Manifest::from_path(cargo_file)
            .unwrap().package.unwrap().name;

        for (name,shader_path) in compile_result.module.unwrap_multi() {
            println!("cargo:rustc-env={}.{}.spv={}", shader_name, name, shader_path.display())
        }
    }

    Ok(())
}
