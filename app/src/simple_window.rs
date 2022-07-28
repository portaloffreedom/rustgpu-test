use std::sync::Arc;
use image::{Frame, ImageBuffer, Rgba};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::device::{Device, Queue};
use vulkano::image::view::ImageView;
use vulkano::shader::ShaderModule;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreationError, Subpass};
use vulkano::format::Format;
use std::default::Default;
use std::convert::TryFrom;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SubpassContents};
use vulkano::image::{ImageAccess, ImageDimensions, StorageImage, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain::{AcquireError, Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError};
use vulkano::{swapchain, sync};
use vulkano::sync::{FlushError, GpuFuture};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use crate::engine::vec::Vertex;

const SHADER_SIMPLE_GRAPHICS_VS: &[u8] = include_bytes!(env!("simple_graphics.main_vs.spv"));
const SHADER_SIMPLE_GRAPHICS_FS: &[u8] = include_bytes!(env!("simple_graphics.main_fs.spv"));

pub fn simple_window(event_loop: EventLoop<()>,
                     device: Arc<Device>,
                     queue: Arc<Queue>,
                     surface: Arc<Surface<Window>>,
                     mut swapchain: Arc<Swapchain<Window>>,
                     images: Vec<Arc<SwapchainImage<Window>>>)
{
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

    let render_pass = get_render_pass(device.clone(), swapchain.clone())
        .unwrap();

    let framebuffers = get_framebuffers(&images, render_pass.clone());

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: surface.window().inner_size().into(),
        depth_range: 0.0..1.0,
    };

    let pipeline = get_pipeline(
        device.clone(),
        r_shader_vs.clone(),
        r_shader_fs.clone(),
        render_pass.clone(),
        viewport.clone(),
    );

    let mut command_buffers = get_command_buffers(
        device.clone(),
        queue.clone(),
        pipeline,
        &framebuffers,
        vertex_buffer.clone()
    );

    // let future = sync::now(device.clone())
    //     .then_execute(queue.clone(), command_buffer)
    //     .unwrap()
    //     .then_signal_fence_and_flush()
    //     .unwrap();
    // future.wait(None).unwrap();

    println!("Rendering image succeded!");

    let mut window_resized = false;
    let mut recreate_swapchain = false;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                window_resized = true;
            }
            Event::RedrawEventsCleared => {
                if window_resized || recreate_swapchain {
                    recreate_swapchain = false;
                    let new_dimensions = surface.window().inner_size();

                    let (new_spapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
                        image_extent: new_dimensions.into(),
                        ..swapchain.create_info()
                    }) {
                        Ok(r) => r,
                        // This error tends to happen when the user is manually resizing the window.
                        // Simply restarting the loop is the easiest way to fix this issue.
                        Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                        Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                    };
                    swapchain = new_spapchain;
                    let new_framebuffers = get_framebuffers(&new_images, render_pass.clone());

                    if window_resized {
                        window_resized = false;

                        viewport.dimensions = new_dimensions.into();
                        let new_pipeline = get_pipeline(
                            device.clone(),
                            r_shader_vs.clone(),
                            r_shader_fs.clone(),
                            render_pass.clone(),
                            viewport.clone()
                        );
                        command_buffers = get_command_buffers(
                            device.clone(),
                            queue.clone(),
                            new_pipeline,
                            &new_framebuffers,
                            vertex_buffer.clone(),
                        );
                    }
                }

                //redraw
                let (image_i, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };
                if suboptimal {
                    recreate_swapchain = true;
                }
                let execution = sync::now(device.clone())
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffers[image_i].clone())
                    .unwrap()
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_i)
                    .then_signal_fence_and_flush();

                match execution {
                    Ok(future) => {
                        future.wait(None).unwrap(); //wait for the gpu to finish
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                    }
                }
            }
            _ => ()
        }
    });
}

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain<Window>>) -> Result<Arc<RenderPass>, RenderPassCreationError> {
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
}

fn get_framebuffers(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view  = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..FramebufferCreateInfo::default()
                },
            ).unwrap()
        })
        .collect::<Vec<_>>()
}

fn get_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main_vs").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        .fragment_shader(fs.entry_point("main_fs").unwrap(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap()
}

fn get_command_buffers(
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                device.clone(),
                queue.family(),
                CommandBufferUsage::MultipleSubmit,
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
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass()
                .unwrap();

            Arc::new(builder.build().unwrap())
        })
        .collect()
}