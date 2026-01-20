use crate::{
    binding::{BindGroupFormat, BindedBuffer, BindedTexture},
    gpu_handle::GpuHandle,
    gpu_vec::GpuVec,
    texture::Texture,
};
use hydrogen_core::global_dep;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;

mod hydrogen {
    pub use hydrogen_core as core;
}

#[derive(Debug, Clone)]
pub struct PipelineDescriptor {
    pub name: &'static str,

    pub shader_source: &'static str,

    pub vertex_shader_entry_point: Option<&'static str>,
    pub vertex_format: &'static [wgpu::VertexFormat],
    pub instance_format: Option<&'static [wgpu::VertexFormat]>,

    pub fragment_shader_entry_point: Option<&'static str>,
    pub target_format: Option<wgpu::TextureFormat>,

    pub bind_groups: &'static [&'static BindGroupFormat],

    pub use_depth: bool,
    pub alpha_to_coverage_enabled: bool,
}

impl Default for PipelineDescriptor {
    fn default() -> Self {
        Self {
            name: "",

            shader_source: "",

            vertex_shader_entry_point: None,
            vertex_format: &[],
            instance_format: None,

            fragment_shader_entry_point: None,
            target_format: None,

            bind_groups: &[],

            use_depth: true,
            alpha_to_coverage_enabled: false,
        }
    }
}

fn generate_vertex_attributes(
    formats: &[wgpu::VertexFormat],
    mut shader_location: u32,
) -> (u64, Vec<wgpu::VertexAttribute>) {
    let mut array_stride = 0u64;

    let mut attributes = Vec::with_capacity(formats.len());
    for format in formats {
        attributes.push(wgpu::VertexAttribute {
            format: *format,
            offset: array_stride,
            shader_location,
        });
        array_stride += format.size();
        shader_location += 1;
    }

    (array_stride, attributes)
}

#[derive(Debug)]
pub struct PipelineBuffers<'a, V, I = u8>
where
    V: bytemuck::NoUninit,
    I: bytemuck::NoUninit,
{
    pub vertices: &'a GpuVec<V>,
    pub instances: Option<&'a GpuVec<I>>,
    pub indices: Option<&'a GpuVec<u32>>,
}

impl<V, I> IntoIterator for PipelineBuffers<'_, V, I>
where
    V: bytemuck::NoUninit,
    I: bytemuck::NoUninit,
{
    type Item = Self;

    type IntoIter = std::iter::Once<Self>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

#[derive(Debug)]
pub struct Pipeline<V, I = u8>
where
    V: bytemuck::NoUninit,
    I: bytemuck::NoUninit,
{
    pub(crate) handle: GpuHandle,
    pub(crate) descriptor: PipelineDescriptor,
    pub(crate) gpu_pipeline: wgpu::RenderPipeline,
    pub(crate) shader_module: wgpu::ShaderModule,

    pub(crate) dummy_vertex_buffer: wgpu::Buffer,
    pub(crate) dummy_instance_buffer: wgpu::Buffer,

    pub(crate) bind_group_layouts: Vec<wgpu::BindGroupLayout>,

    pub(crate) _phantom: PhantomData<(V, I)>,
}

impl<V, I> Pipeline<V, I>
where
    V: bytemuck::NoUninit,
    I: bytemuck::NoUninit,
{
    pub fn new(descriptor: PipelineDescriptor) -> Self {
        let handle = global_dep!(GpuHandle).clone();

        let shader_module = handle
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(descriptor.name),
                source: wgpu::ShaderSource::Wgsl(descriptor.shader_source.into()),
            });

        let (vertex_stride, vertex_attributes) =
            generate_vertex_attributes(descriptor.vertex_format, 0);
        let (instance_stride, instance_attributes) =
            if let Some(instance_format) = descriptor.instance_format {
                generate_vertex_attributes(instance_format, vertex_attributes.len() as u32)
            } else {
                (0u64, vec![])
            };

        let bind_group_layouts = descriptor
            .bind_groups
            .iter()
            .map(|&format| handle.create_bind_group_layout(format))
            .collect::<Vec<wgpu::BindGroupLayout>>();

        let gpu_pipeline = handle
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(descriptor.name),
                layout: Some(
                    &handle
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some(descriptor.name),
                            bind_group_layouts: &bind_group_layouts
                                .iter()
                                .collect::<Vec<&wgpu::BindGroupLayout>>(),
                            push_constant_ranges: &[],
                        }),
                ),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: descriptor.vertex_shader_entry_point,
                    compilation_options: Default::default(),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: vertex_stride,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &vertex_attributes,
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: instance_stride,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &instance_attributes,
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: descriptor.use_depth.then_some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: descriptor.use_depth,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: descriptor.alpha_to_coverage_enabled,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: descriptor.fragment_shader_entry_point,
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: descriptor
                            .target_format
                            .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });

        let dummy_vertex_buffer =
            handle
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("'{}' Dummy Vertex Buffer", descriptor.name)),
                    contents: &vec![0u8; (vertex_stride as usize).max(1)],
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let dummy_instance_buffer =
            handle
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("'{}' Dummy Instance Buffer", descriptor.name)),
                    contents: &vec![0u8; (instance_stride as usize).max(1)],
                    usage: wgpu::BufferUsages::VERTEX,
                });

        Self {
            handle,
            descriptor,
            gpu_pipeline,
            shader_module,

            dummy_vertex_buffer,
            dummy_instance_buffer,

            bind_group_layouts,

            _phantom: PhantomData,
        }
    }

    pub fn create_bind_group(
        &self,
        group_layout_index: usize,
        resources: Vec<wgpu::BindingResource>,
    ) -> wgpu::BindGroup {
        self.handle
            .create_bind_group(&self.bind_group_layouts[group_layout_index], resources)
    }

    pub fn binded_texture(&self, group_layout_index: usize, texture: Texture) -> BindedTexture {
        self.handle
            .binded_texture(&self.bind_group_layouts[group_layout_index], texture)
    }

    pub fn binded_buffer<T>(&self, group_layout_index: usize, buffer: GpuVec<T>) -> BindedBuffer<T>
    where
        T: bytemuck::NoUninit,
    {
        self.handle
            .binded_buffer(&self.bind_group_layouts[group_layout_index], buffer)
    }
}
