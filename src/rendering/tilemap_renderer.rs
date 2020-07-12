use super::utils;
use super::SWAPCHAIN_FORMAT;

pub struct TileMapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    mesh_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,
    tileset_bind_group_layout: wgpu::BindGroupLayout,
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
                wgpu::BindGroupLayoutEntry::new(
                    2,
                    wgpu::ShaderStage::FRAGMENT,
                    wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });

        let vs_shader = device.create_shader_module(wgpu::include_spirv!("../../shaders/tile.vert.spv"));
        let fs_shader = device.create_shader_module(wgpu::include_spirv!("../../shaders/tile.frag.spv"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &mesh_bind_group_layout,
                &tileset_bind_group_layout,
                &mesh_bind_group_layout,
            ],
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
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
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

        let camera_bind_group =
            create_camera_bind_group(&mesh_bind_group_layout, device, &glm::identity(), &glm::identity());

        let tileset_bind_group = create_tileset_bind_group(
            &tileset_bind_group_layout,
            device,
            utils::rgba_null_texture(device, queue),
            &[1, 1],
        );

        Self {
            render_pipeline,
            mesh_bind_group_layout,
            camera_bind_group,
            tileset_bind_group_layout,
            tileset_bind_group,
        }
    }

    pub fn update_camera(&mut self, device: &wgpu::Device, view: &glm::Mat4, projection: &glm::Mat4) {
        self.camera_bind_group = create_camera_bind_group(&self.mesh_bind_group_layout, device, view, projection);
    }

    pub fn update_tileset(&mut self, device: &wgpu::Device, texture_view: &wgpu::TextureView, size: &[u32; 2]) {
        self.tileset_bind_group =
            create_tileset_bind_group(&self.tileset_bind_group_layout, device, texture_view, size);
    }

    pub fn create_chunk_bind_group(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.mesh_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.slice(..)),
            }],
            label: None,
        })
    }

    pub fn start<'a, 'p>(&'a self, pass: &'p mut wgpu::RenderPass<'a>) -> TileMapRendererPass<'a, 'p> {
        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
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
    pub fn draw_chunk(&mut self, data: &'a wgpu::BindGroup) {
        self.pass.set_bind_group(2, data, &[]);
        self.pass.draw(0..4, 0..256);
    }
}

fn create_camera_bind_group(
    layout: &wgpu::BindGroupLayout,
    device: &wgpu::Device,
    view: &glm::Mat4,
    projection: &glm::Mat4,
) -> wgpu::BindGroup {
    let mut data = [0f32; 16 * 2];
    data[..16].copy_from_slice(view.as_slice());
    data[16..].copy_from_slice(projection.as_slice());
    create_mesh_bind_group(layout, device, bytemuck::cast_slice(&data))
}

fn create_mesh_bind_group(layout: &wgpu::BindGroupLayout, device: &wgpu::Device, data: &[u8]) -> wgpu::BindGroup {
    let chunk_uniform_buffer =
        device.create_buffer_with_data(data, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST);

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(chunk_uniform_buffer.slice(..)),
        }],
        label: None,
    })
}

fn create_tileset_bind_group(
    layout: &wgpu::BindGroupLayout,
    device: &wgpu::Device,
    texture_view: &wgpu::TextureView,
    size: &[u32; 2],
) -> wgpu::BindGroup {
    let tileset_uniform_buffer = device.create_buffer_with_data(
        bytemuck::cast_slice(size),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        bindings: &[
            wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::Binding {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(utils::pixel_sampler(device)),
            },
            wgpu::Binding {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(tileset_uniform_buffer.slice(..)),
            },
        ],
        label: None,
    })
}
