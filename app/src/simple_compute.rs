use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::pipeline::{ComputePipeline, PipelineBindPoint, Pipeline};
use vulkano::shader::ShaderModule;
use vulkano::sync;
use vulkano::sync::GpuFuture;

const SHADER_SIMPLE_COMPUTE: &[u8] = include_bytes!(env!("simple_compute.main_cs.spv"));

pub fn simple_compute(device: Arc<Device>, queue: Arc<Queue>) {
    // load shader
    assert_eq!(SHADER_SIMPLE_COMPUTE.len() % 4, 0);
    let shader = unsafe {
        ShaderModule::from_bytes(device.clone(), SHADER_SIMPLE_COMPUTE)
            .unwrap()
    };

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        shader.entry_point("main_cs").unwrap(),
        &(),
        None,
        |_| {})
        .expect("failed to create compute pipeline");

    // data buffer shader -----------------------------------------------------
    let data_iter = 0..65536;
    let data_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, data_iter)
            .expect("failed to create buffer");

    let layout = compute_pipeline.layout().set_layouts()
        .get(0)
        .unwrap();
    let set = PersistentDescriptorSet::new(
        layout.clone(),
        [WriteDescriptorSet::buffer(0, data_buffer.clone())], // 0 is the binding
    )
        .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
        .unwrap();

    builder
        .bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            0,
            set,
        )
        .dispatch([1024, 1, 1])
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }

    println!("Everything succeded!");
}