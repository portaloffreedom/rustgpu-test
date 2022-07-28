extern crate core;

use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, DeviceExtensions};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::image::ImageUsage;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::{SurfaceInfo, Swapchain, SwapchainCreateInfo};
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::CursorIcon::Default;
use winit::window::WindowBuilder;

mod fractal;
mod simple_compute;
mod simple_graphics;
mod simple_window;
pub mod engine;

fn main() {
    let required_extensions = vulkano_win::required_extensions();

    let instance = Instance::new(InstanceCreateInfo {
        enabled_extensions: required_extensions,
        ..InstanceCreateInfo::default()
    })
        .expect("failed to create instance");

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };

    let (physical_device, queue_family)
        = engine::select_physical_device(&instance, surface.clone(), &device_extensions);

    for family in physical_device.queue_families() {
        println!("Found a queue family with {:?} queue(s)", family.queues_count());
    }

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            enabled_extensions: physical_device
                .required_extensions()
                .union(&device_extensions),
            ..DeviceCreateInfo::default()
        })
        .expect("failed to create vulkan device");

    let queue = queues.next().unwrap();

    let caps = physical_device
        .surface_capabilities(&surface, SurfaceInfo::default())
        .expect("failed to get surface capabilities");

    let dimensions = surface.window().inner_size();
    let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
    let image_format = Some(
        physical_device
            .surface_formats(&surface, SurfaceInfo::default())
            .unwrap()[0]
            .0,
    );

    let (swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage::color_attachment(),
            composite_alpha,
            ..SwapchainCreateInfo::default()
        }
    ).unwrap();

    // simple compute shader -----------------------------------------------------
    simple_compute::simple_compute(device.clone(), queue.clone());

    // image compute shader ------------------------------------------------------
    // crashes, so disabled
    //fractal::fractal(device.clone(), queue.clone());

    // render a triangle ---------------------------------------------------------
    //simple_graphics::simple_graphics(device.clone(), queue.clone());

    // render a triangle into a window -------------------------------------------
    simple_window::simple_window(event_loop, device.clone(), queue.clone(), surface, swapchain.clone(), images);
}
