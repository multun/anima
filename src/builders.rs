use std::borrow::Cow::Borrowed;

pub async fn request_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropiate adapter")
}

pub async fn request_default_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        )
        .await
        .expect("Failed to create device")
}

pub struct SwapChainDescBuilder {
    usage: wgpu::TextureUsage,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    present_mode: wgpu::PresentMode,
}

impl SwapChainDescBuilder {
    pub fn new(width: u32, height: u32, usage: wgpu::TextureUsage) -> SwapChainDescBuilder {
        SwapChainDescBuilder {
            usage,
            width,
            height,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::Mailbox,
        }
    }

    pub fn build(self) -> wgpu::SwapChainDescriptor {
        wgpu::SwapChainDescriptor {
            usage: self.usage,
            format: self.format,
            width: self.width,
            height: self.height,
            present_mode: self.present_mode,
        }
    }
}

pub struct ColorStateDescBuilder {
    format: wgpu::TextureFormat,
    color_blend: wgpu::BlendDescriptor,
    alpha_blend: wgpu::BlendDescriptor,
    write_mask: wgpu::ColorWrite,
}

impl ColorStateDescBuilder {
    pub fn new(format: wgpu::TextureFormat) -> Self {
        Self {
            format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }
    }

    pub fn format(&mut self, format: wgpu::TextureFormat) -> &mut Self {
        self.format = format;
        self
    }

    pub fn color_blend(mut self, color_blend: wgpu::BlendDescriptor) -> Self {
        self.color_blend = color_blend;
        self
    }

    pub fn alpha_blend(mut self, alpha_blend: wgpu::BlendDescriptor) -> Self {
        self.alpha_blend = alpha_blend;
        self
    }

    pub fn build(self) -> wgpu::ColorStateDescriptor {
        wgpu::ColorStateDescriptor {
            format: self.format,
            color_blend: self.color_blend,
            alpha_blend: self.alpha_blend,
            write_mask: self.write_mask,
        }
    }
}

pub struct VertexStateDescBuilder<'a> {
    index_format: wgpu::IndexFormat,
    vertex_buffers: std::borrow::Cow<'a, [wgpu::VertexBufferDescriptor<'a>]>,
}

impl<'a> VertexStateDescBuilder<'a> {
    pub fn default() -> Self {
        Self {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: Borrowed(&[]),
        }
    }

    pub fn build(self) -> wgpu::VertexStateDescriptor<'a> {
        wgpu::VertexStateDescriptor {
            index_format: self.index_format,
            vertex_buffers: self.vertex_buffers,
        }
    }
}

pub struct RenderPipelineDescBuilder<'a> {
    layout: &'a wgpu::PipelineLayout,
    vertex_stage: wgpu::ProgrammableStageDescriptor<'a>,
    fragment_stage: Option<wgpu::ProgrammableStageDescriptor<'a>>,
    primitive_topology: wgpu::PrimitiveTopology,
    color_states: std::borrow::Cow<'a, [wgpu::ColorStateDescriptor]>,
    vertex_state: wgpu::VertexStateDescriptor<'a>,
    alpha_to_coverage_enabled: bool,
    rasterization_state: Option<wgpu::RasterizationStateDescriptor>,
    sample_count: u32,
    sample_mask: u32,
}

impl<'a> RenderPipelineDescBuilder<'a> {
    pub fn new(
        layout: &'a wgpu::PipelineLayout,
        vertex_stage: wgpu::ProgrammableStageDescriptor<'a>,
    ) -> Self {
        Self {
            layout,
            vertex_stage,
            fragment_stage: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: Borrowed(&[]),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: Borrowed(&[]),
            },
            alpha_to_coverage_enabled: false,
            rasterization_state: None,
            sample_count: 1,
            sample_mask: !0,
        }
    }

    pub fn color_states(mut self, color_states: &'a [wgpu::ColorStateDescriptor]) -> Self {
        self.color_states = Borrowed(&color_states);
        self
    }

    pub fn fragment_stage(mut self, fragment_stage: wgpu::ProgrammableStageDescriptor<'a>) -> Self {
        self.fragment_stage = Some(fragment_stage);
        self
    }

    pub fn sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    pub fn sample_mask(mut self, sample_mask: u32) -> Self {
        self.sample_mask = sample_mask;
        self
    }

    pub fn alpha_to_coverage_enabled(mut self, alpha_to_coverage_enabled: bool) -> Self {
        self.alpha_to_coverage_enabled = alpha_to_coverage_enabled;
        self
    }

    pub fn build(self) -> wgpu::RenderPipelineDescriptor<'a> {
        wgpu::RenderPipelineDescriptor {
            layout: &self.layout,
            vertex_stage: self.vertex_stage,
            fragment_stage: self.fragment_stage,
            rasterization_state: self.rasterization_state,
            primitive_topology: self.primitive_topology,
            color_states: self.color_states,
            depth_stencil_state: None,
            vertex_state: self.vertex_state,
            sample_count: self.sample_count,
            sample_mask: self.sample_mask,
            alpha_to_coverage_enabled: self.alpha_to_coverage_enabled,
        }
    }
}
