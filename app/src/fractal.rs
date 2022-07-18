use std::sync::Arc;
use image::{ImageBuffer, Rgba};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::format::ClearValue;
use vulkano::format::Format;
use vulkano::image::{ImageDimensions, StorageImage};
use vulkano::image::view::ImageView;
use vulkano::image::ImageAccess;
use vulkano::pipeline::{ComputePipeline, PipelineBindPoint};
use vulkano::pipeline::Pipeline;
use vulkano::shader::ShaderModule;
use vulkano::sync;
use vulkano::sync::GpuFuture;

const SHADER_FRACTAL: &[u8] = include_bytes!(env!("fractal.fractal.spv"));

#[allow(dead_code)]
pub fn fractal(device: Arc<Device>, queue: Arc<Queue>) {
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

    let image_dimensions = image.dimensions();
    let image_dimensions_buf = [image_dimensions.width(), image_dimensions.height()];

    let image_view = ImageView::new_default(image.clone()).unwrap();

    let image_size_vkbuffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::all(),
        false,
        image_dimensions_buf,
    )
        .expect("failed to create image size buffer");


    assert_eq!(SHADER_FRACTAL.len() % 4, 0);
    let fractal_shader = unsafe {
        ShaderModule::from_bytes(device.clone(), SHADER_FRACTAL)
            .unwrap()
    };

    let compute_fractal = ComputePipeline::new(
        device.clone(),
        fractal_shader.entry_point("fractal").unwrap(),
        &(),
        None,
        |_| {},
    )
        .expect("failed to create compute pipeline");

    let layout = compute_fractal.layout().set_layouts()
        .get(0)
        .unwrap();
    let set = PersistentDescriptorSet::new(
        layout.clone(),
        [
            WriteDescriptorSet::image_view(0, image_view.clone()),
            WriteDescriptorSet::buffer(1, image_size_vkbuffer.clone()), // 0 is the binding
        ],
    )
        .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
        .unwrap();

    let buf = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        (0..image_dimensions.width() * image_dimensions.height() * 4).map(|_| 0u8),
    )
        .expect("failed to create buffer");

    builder
        .clear_color_image(image.clone(), ClearValue::Float([0.0, 0.0, 1.0, 1.0]))
        .unwrap()
        .bind_pipeline_compute(compute_fractal.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_fractal.layout().clone(),
            0,
            set,
        )
        .dispatch([image_dimensions.width() / 8, image_dimensions.height() / 8, 1])
        .unwrap()
        .copy_image_to_buffer(image.clone(), buf.clone())
        .unwrap();

    let command_buffer_2 = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer_2)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();

    image.save("image.png").unwrap();

    println!("Image stuff succeded!");
}