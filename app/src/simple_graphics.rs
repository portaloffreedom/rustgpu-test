use std::sync::Arc;
use image::{ImageBuffer, Rgba};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::{Device, Queue};
use vulkano::image::view::ImageView;
use vulkano::shader::ShaderModule;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::format::Format;
use std::default::Default;
use std::convert::TryFrom;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::image::{ImageDimensions, StorageImage};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use crate::engine::vec::Vertex;

const SHADER_SIMPLE_GRAPHICS_VS: &[u8] = include_bytes!(env!("simple_graphics.main_vs.spv"));
const SHADER_SIMPLE_GRAPHICS_FS: &[u8] = include_bytes!(env!("simple_graphics.main_fs.spv"));

mod glsl_vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}

mod glsl_fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}"
    }
}

pub fn simple_graphics(device: Arc<Device>, queue: Arc<Queue>) {
    println!("Rendering image started!");
    assert_eq!(SHADER_SIMPLE_GRAPHICS_VS.len() % 4, 0);
    assert_eq!(SHADER_SIMPLE_GRAPHICS_FS.len() % 4, 0);
    let r_shader_vs = unsafe {
        ShaderModule::from_bytes(device.clone(), SHADER_SIMPLE_GRAPHICS_VS)
            .unwrap()
    };
    let r_shader_fs = unsafe {
        ShaderModule::from_bytes(device.clone(), SHADER_SIMPLE_GRAPHICS_FS)
            .unwrap()
    };

    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [ 0.0,  0.5] };
    let vertex3 = Vertex { position: [ 0.5, -0.25] };

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::vertex_buffer(),
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    )
        .unwrap();

    let render_pass = vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: Format::R8G8B8A8_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
        .unwrap();


    let image = StorageImage::new(
        device.clone(),
        ImageDimensions::Dim2d {
            width: 1024,
            height: 1024,
            array_layers: 1, // images can be arrays of layers
        },
        Format::R8G8B8A8_UNORM,
        Some(queue.family()),
    )
        .unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();
    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![view],
            ..Default::default()
        },
    )
        .unwrap();

    let buf = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
        .expect("failed to create buffer");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [1024.0, 1024.0],
        depth_range: 0.0..1.0,
    };

    let glsl_vs = glsl_vs::load(device.clone()).expect("failed to create shader module");
    let glsl_fs = glsl_fs::load(device.clone()).expect("failed to create shader module");

    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(r_shader_vs.entry_point("main_vs").unwrap(), ())
        // .vertex_shader(glsl_vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        // .fragment_shader(r_shader_fs.entry_point("main_fs").unwrap(), ())
        .fragment_shader(glsl_fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
        .unwrap();
    builder
        .begin_render_pass(
            framebuffer.clone(),
            SubpassContents::Inline,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        )
        .unwrap()
        .bind_pipeline_graphics(pipeline.clone())
        .bind_vertex_buffers(0, vertex_buffer.clone())
        .draw(3,1,0,0) // 3 is the number of vertices, 1 is the number of instances
        .unwrap()
        .end_render_pass()
        .unwrap()
        .copy_image_to_buffer(image, buf.clone())
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();

    println!("Rendering finished, saving file");

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();

    println!("Rendering image succeded!");
}