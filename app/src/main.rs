const SHADER_BYTES: &[u8] = include_bytes!(env!("myshader.spv"));

use vulkano::device::{Device, DeviceCreateInfo, Features, QueueCreateInfo};
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::shader::ShaderModule;
use vulkano::sync;
use vulkano::sync::GpuFuture;

fn main() {
    let instance = Instance::new(InstanceCreateInfo::default())
        .expect("failed to create instance");

    let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

    for family in physical_device.queue_families() {
        println!("Found a queue family {:?} with {:?} queue(s)", family, family.queues_count());
    }

    let queue_family = physical_device.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            ..Default::default()
        })
        .expect("failed to create vulkan device");

    let queue = queues.next().unwrap();

    let data: i32 = 12;

    let buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, data)
        .expect("failed to create buffer");

    let data_iter = 0..65536;
    let data_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, data_iter)
            .expect("failed to create buffer");

    assert_eq!(SHADER_BYTES.len() % 4, 0);
    let shader = unsafe {
        ShaderModule::from_bytes(device.clone(), SHADER_BYTES)
            .unwrap()
    };

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        shader.entry_point("main_cs").unwrap(),
        &(),
        None,
        |_| {})
        .expect("failed to create compute pipeline");

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
