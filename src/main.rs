mod box_renderer;
mod framework;
mod fxaa;

use box_renderer::BoxRenderer;
use fxaa::FXAAPass;

struct Example {
    box_renderer: BoxRenderer,
    fxaa: FXAAPass,
    format: wgpu::TextureFormat,
    fxaa_input: wgpu::Texture,
}

impl Example {
    fn regen_buffers(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32
    ) -> wgpu::Texture {
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        device.create_texture(&wgpu::TextureDescriptor {
            format,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            label: None,
        })
    }
}

impl framework::Example for Example {

    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let width = sc_desc.width;
        let height = sc_desc.height;
        let format = sc_desc.format;

        Example {
            format,
            box_renderer: BoxRenderer::new(format, width, height, device, queue),
            fxaa: FXAAPass::new(format, width, height, device, queue),
            fxaa_input: Example::regen_buffers(device, format, width, height),
        }
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let width = sc_desc.width;
        let height = sc_desc.height;
        self.box_renderer.resize(width, height, device, queue);
        self.fxaa.resize(width, height, device, queue);
        self.fxaa_input = Example::regen_buffers(device, self.format, width, height);
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &impl futures::task::LocalSpawn,
    ) {
        let fxaa_input = self.fxaa_input.create_default_view();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.box_renderer
            .render(&fxaa_input, device, queue, &mut encoder);

        self.fxaa.render(&frame.view, &fxaa_input, device, queue, &mut encoder);
        queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    env_logger::init();
    framework::run::<Example>("cube");
}
