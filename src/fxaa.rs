use bytemuck::{Pod, Zeroable};
use std::borrow::Cow::Borrowed;
use wgpu::util::DeviceExt;

pub struct FXAAPass {
    // volatile bind group (0)
    volatile_bind_group_layout: wgpu::BindGroupLayout,
    volatile_bind_group: Option<wgpu::BindGroup>,

    // rarely changed bind group (1)
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,

    pipeline: wgpu::RenderPipeline,

    width: u32,
    height: u32,
}

impl FXAAPass {
    pub fn volatile_bind_group(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        source_image: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let pixel_size = &[1. / (width as f32), 1. / (height as f32)];
        let pixel_size_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pixel Size Buffer"),
            contents: bytemuck::cast_slice(pixel_size),
            usage: wgpu::BufferUsage::UNIFORM,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(Borrowed("Volatile bind group")),
            layout: &self.volatile_bind_group_layout,
            entries: Borrowed(&[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_image),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(pixel_size_buf.slice(..)),
                },
            ]),
        })
    }

    pub fn new(format: wgpu::TextureFormat, width: u32, height: u32, device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        // the bind group that changes on each frame
        let volatile_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: Borrowed(&[
                    // t_screenTexture: texture2D
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::SampledTexture {
                            // This is msaa related, and has nothing to do with the number of times the texture
                            // is sampled inside the shader.
                            multisampled: false,
                            component_type: wgpu::TextureComponentType::Float,
                            dimension: wgpu::TextureViewDimension::D2Array,
                        },
                    ),
                    // pixel_size: vec2
                    wgpu::BindGroupLayoutEntry::new(
                        1,
                        wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(4 * 2),
                        },
                    ),
                ]),
            });

        // the bind group that never changes
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: Borrowed(&[wgpu::BindGroupLayoutEntry::new(
                0,
                wgpu::ShaderStage::FRAGMENT,
                wgpu::BindingType::Sampler { comparison: false },
            )]),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: Borrowed(&[&volatile_bind_group_layout, &bind_group_layout]),
            push_constant_ranges: Borrowed(&[]),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: Borrowed(&[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            }]),
            label: None,
        });

        // Create the render pipeline
        let vs_module = device.create_shader_module(wgpu::include_spirv!("fxaa.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("fxaa.frag.spv"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: Borrowed("main"),
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: Borrowed("main"),
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: Borrowed(&[wgpu::ColorStateDescriptor {
                format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }]),
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: Borrowed(&[]),
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        // Done
        FXAAPass {
            volatile_bind_group_layout,
            volatile_bind_group: None,
            bind_group,
            sampler,
            pipeline,
            width,
            height,
        }
    }

    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.width = width;
        self.height = height;
    }

    pub fn render(
        &mut self,
        dest_image: &wgpu::TextureView,
        source_image: &wgpu::TextureView,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.volatile_bind_group = Some(self.volatile_bind_group(device, self.width, self.height, source_image));

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: Borrowed(&[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: dest_image,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }]),
            depth_stencil_attachment: None,
        });
        rpass.push_debug_group("Prepare FXAA resources.");
        rpass.set_pipeline(&self.pipeline);
        if let Some(ref bind_group) = self.volatile_bind_group {
            rpass.set_bind_group(0, bind_group, &[]);
        }
        rpass.set_bind_group(1, &self.bind_group, &[]);
        rpass.pop_debug_group();
        rpass.insert_debug_marker("FXAA!");
        rpass.draw(0..4, 0..1);
    }
}
