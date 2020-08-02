use std::borrow::Cow::Borrowed;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub mod builders;

use builders::*;

fn empty_pipeline_layout_desc<'a>() -> wgpu::PipelineLayoutDescriptor<'a> {
    wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: Borrowed(&[]),
        push_constant_ranges: Borrowed(&[]),
    }
}

fn prog_stage_desc<'a>(
    module: &'a wgpu::ShaderModule,
    name: &'static str,
) -> wgpu::ProgrammableStageDescriptor<'a> {
    wgpu::ProgrammableStageDescriptor {
        module,
        entry_point: Borrowed(name),
    }
}

async fn run(event_loop: EventLoop<()>, window: Window, swapchain_format: wgpu::TextureFormat) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = request_adapter(&instance, &surface).await;

    // Create the logical device and command queue
    let (device, queue) = request_default_device(&adapter).await;

    let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
    let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

    let pipeline_layout = device.create_pipeline_layout(&empty_pipeline_layout_desc());

    let color_states = [ColorStateDescBuilder::new(swapchain_format).build()];
    let render_pipeline_desc =
        RenderPipelineDescBuilder::new(&pipeline_layout, prog_stage_desc(&vs_module, "main"))
            .color_states(&color_states)
            .fragment_stage(prog_stage_desc(&fs_module, "main"))
            .build();
    let render_pipeline = device.create_render_pipeline(&render_pipeline_desc);

    let mut sc_desc = SwapChainDescBuilder::new(
        size.width,
        size.height,
        wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    )
    .build();
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (
            &instance,
            &adapter,
            &vs_module,
            &fs_module,
            &pipeline_layout,
        );

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Recreate the swap chain with the new size
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture")
                    .output;
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: Borrowed(&[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        }]),
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(feature = "subscriber")]
    wgpu::util::initialize_default_subscriber(None);

    // Temporarily avoid srgb formats for the swapchain on the web
    futures::executor::block_on(run(event_loop, window, wgpu::TextureFormat::Bgra8UnormSrgb));
}
