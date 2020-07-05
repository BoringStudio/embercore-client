use super::utils;
use super::SWAPCHAIN_FORMAT;

pub struct TileMapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    mesh_bind_group: wgpu::BindGroup,
    tileset_bind_group: wgpu::BindGroup,
}

impl TileMapRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mesh_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[wgpu::BindGroupLayoutEntry::new(
                0,
                wgpu::ShaderStage::VERTEX,
                wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
            )],
        });

        let tileset_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry::new(
                    0,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                    },
                ),
                wgpu::BindGroupLayoutEntry::new(
                    1,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::Sampler { comparison: false },
                ),
            ],
        });

        let mesh_uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&create_uniform_data(&glm::identity(), &glm::identity())),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let vs_shader = device.create_shader_module(wgpu::include_spirv!("../../shaders/tile.vert.spv"));
        let fs_shader = device.create_shader_module(wgpu::include_spirv!("../../shaders/tile.frag.spv"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&mesh_bind_group_layout, &tileset_bind_group_layout],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_shader,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_shader,
                entry_point: "main",
            }),
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: SWAPCHAIN_FORMAT,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let mesh_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mesh_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(mesh_uniform_buffer.slice(..)),
            }],
            label: None,
        });

        let tileset_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &tileset_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(utils::rgba_null_texture(device, queue)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(utils::pixel_sampler(device)),
                },
            ],
            label: None,
        });

        Self {
            render_pipeline,
            mesh_bind_group,
            tileset_bind_group,
        }
    }

    pub fn start<'a, 'p>(&'a self, pass: &'p mut wgpu::RenderPass<'a>) -> TileMapRendererPass<'a, 'p> {
        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &self.mesh_bind_group, &[]);
        pass.set_bind_group(1, &self.tileset_bind_group, &[]);

        TileMapRendererPass { renderer: self, pass }
    }
}

pub struct TileMapRendererPass<'a, 'p> {
    renderer: &'a TileMapRenderer,
    pass: &'p mut wgpu::RenderPass<'a>,
}

impl<'a, 'e> TileMapRendererPass<'a, 'e> {
    #[inline]
    pub fn draw_tile(&mut self) {
        self.pass.draw(0..4, 0..1);
    }
}

pub fn create_uniform_data(projection: &glm::Mat4, view: &glm::Mat4) -> [f32; 32] {
    let mut raw = [0f32; 16 * 2];
    raw[..16].copy_from_slice(projection.as_slice());
    raw[16..].copy_from_slice(view.as_slice());
    raw
}
