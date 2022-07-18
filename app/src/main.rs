
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::{Instance, InstanceCreateInfo};

mod fractal;
mod simple_compute;
mod simple_graphics;
pub mod engine;

fn main() {
    let instance = Instance::new(InstanceCreateInfo::default())
        .expect("failed to create instance");

    let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

    for family in physical_device.queue_families() {
        println!("Found a queue family with {:?} queue(s)", family.queues_count());
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

    // simple compute shader -----------------------------------------------------
    simple_compute::simple_compute(device.clone(), queue.clone());

    // image compute shader ------------------------------------------------------
    // crashes, so disabled
    // fractal::fractal(device.clone(), queue.clone());

    // render a triangle ---------------------------------------------------------
    simple_graphics::simple_graphics(device.clone(), queue.clone());
}
