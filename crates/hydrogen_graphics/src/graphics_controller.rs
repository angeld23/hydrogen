use crate::{
    color::RGBA,
    gpu_handle::GpuHandle,
    gpu_vec::GpuVec,
    pipeline::{Pipeline, PipelineBuffers, PipelineDescriptor},
    render_target::RenderTarget,
    shaders::SHADER_PRESENT,
    texture::Texture,
    vertex::Vertex2D,
};
use anyhow::Result;
use hydrogen_math::bbox;
use std::{collections::BTreeMap, rc::Rc, sync::Arc};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct GraphicsController {
    handle: GpuHandle,

    window_surface: wgpu::Surface<'static>,
    window_surface_config: wgpu::SurfaceConfiguration,
    window_size: PhysicalSize<u32>,
    default_present_mode: wgpu::PresentMode,

    present_pipeline: Option<Pipeline<Vertex2D>>,
    present_vertices: GpuVec<Vertex2D>,
    present_indices: GpuVec<u32>,

    render_targets: BTreeMap<&'static str, Rc<RenderTarget>>,
}

impl GraphicsController {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let window_surface = instance.create_surface(Arc::clone(&window))?;
        let adapter = futures::executor::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&window_surface),
            },
        ))?;

        let (device, queue) =
            futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::CLEAR_TEXTURE,
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            }))?;

        let window_size = window.inner_size();
        let window_surface_capabilities = window_surface.get_capabilities(&adapter);
        let window_surface_format = window_surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(window_surface_capabilities.formats[0]);

        let default_present_mode = window_surface_capabilities.present_modes[0];
        let window_surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: window_surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: default_present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: window_surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        window_surface.configure(&device, &window_surface_config);

        let handle = GpuHandle { device, queue };

        let present_vertices = GpuVec::new(
            &handle,
            wgpu::BufferUsages::VERTEX,
            Vertex2D::fill_screen(RGBA::WHITE, bbox!([0.0, 0.0], [1.0, 1.0])).to_vec(),
        );
        let present_indices =
            GpuVec::new(&handle, wgpu::BufferUsages::INDEX, vec![0, 1, 2, 2, 3, 0]);

        let mut controller = Self {
            handle,

            window_surface,
            window_surface_config,
            window_size,
            default_present_mode,

            present_pipeline: None,
            present_vertices,
            present_indices,

            render_targets: BTreeMap::new(),
        };

        controller.present_pipeline = Some(Pipeline::new(
            &controller,
            PipelineDescriptor {
                name: "Present to Screen",
                shader_source: SHADER_PRESENT,
                vertex_format: Vertex2D::VERTEX_FORMAT,
                instance_format: None,
                target_format: Some(window_surface_format),
                bind_groups: &[Texture::STANDARD_BIND_GROUP_LAYOUT],
                use_depth: false,
                alpha_to_coverage_enabled: false,

                ..Default::default()
            },
        ));

        Ok(controller)
    }

    pub fn handle(&self) -> &GpuHandle {
        &self.handle
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width * new_size.height == 0 {
            return;
        }

        self.window_size = new_size;
        self.window_surface_config.width = new_size.width;
        self.window_surface_config.height = new_size.height;
        self.window_surface
            .configure(&self.handle.device, &self.window_surface_config);
    }

    pub fn is_vsync_enabled(&self) -> bool {
        self.window_surface_config.present_mode != wgpu::PresentMode::AutoNoVsync
    }

    pub fn set_vsync_enabled(&mut self, enabled: bool) {
        if self.is_vsync_enabled() == enabled {
            return;
        }

        self.window_surface_config.present_mode = if enabled {
            self.default_present_mode
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.window_surface
            .configure(&self.handle.device, &self.window_surface_config);
    }

    pub fn window_surface_format(&self) -> wgpu::TextureFormat {
        self.window_surface_config.format
    }

    pub fn present_to_screen(&self, texture: &Texture) -> Result<()> {
        let output = self.window_surface.get_current_texture()?;
        let output_view = output.texture.create_view(&Default::default());

        self.internal_render(
            &output_view,
            None,
            false,
            false,
            self.present_pipeline.as_ref().unwrap(),
            [PipelineBuffers {
                vertices: &self.present_vertices,
                instances: None,
                indices: Some(&self.present_indices),
            }],
            [&self.present_pipeline.as_ref().unwrap().create_bind_group(
                0,
                vec![
                    wgpu::BindingResource::TextureView(&texture.view),
                    wgpu::BindingResource::Sampler(&texture.sampler),
                ],
            )],
        );

        output.present();

        Ok(())
    }

    /// ### Returns
    ///
    /// (`was_recreated`, `render_target_pointer`)
    pub fn render_target(
        &mut self,
        name: &'static str,
        width: u32,
        height: u32,
    ) -> (bool, Rc<RenderTarget>) {
        let recreate = match self.render_targets.get(name) {
            Some(target) => target.width() != width || target.height() != height,
            None => true,
        };

        if recreate {
            self.render_targets.insert(
                name,
                Rc::new(RenderTarget::new(
                    &self.handle,
                    Texture::new(
                        &self.handle,
                        &wgpu::TextureDescriptor {
                            label: Some(name),
                            size: wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            usage: wgpu::TextureUsages::COPY_DST
                                | wgpu::TextureUsages::COPY_SRC
                                | wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::RENDER_ATTACHMENT,
                            view_formats: &[],
                        },
                        &wgpu::SamplerDescriptor::default(),
                    ),
                )),
            );
        }

        (recreate, Rc::clone(self.render_targets.get(name).unwrap()))
    }

    pub fn window_sized_render_target(&mut self, name: &'static str) -> (bool, Rc<RenderTarget>) {
        self.render_target(name, self.window_size.width, self.window_size.height)
    }

    pub fn vec<T>(&self, contents: Vec<T>, usage: wgpu::BufferUsages) -> GpuVec<T>
    where
        T: bytemuck::NoUninit,
    {
        GpuVec::new(&self.handle, usage, contents)
    }

    pub fn vertex_vec<T>(&self, contents: Vec<T>) -> GpuVec<T>
    where
        T: bytemuck::NoUninit,
    {
        self.vec(contents, wgpu::BufferUsages::VERTEX)
    }

    pub fn index_vec<T>(&self, contents: Vec<T>) -> GpuVec<T>
    where
        T: bytemuck::NoUninit,
    {
        self.vec(contents, wgpu::BufferUsages::INDEX)
    }

    pub fn uniform_vec<T>(&self, contents: Vec<T>) -> GpuVec<T>
    where
        T: bytemuck::NoUninit,
    {
        self.vec(contents, wgpu::BufferUsages::UNIFORM)
    }

    pub fn render<V, I>(
        &self,
        target: &RenderTarget,
        pipeline: &Pipeline<V, I>,
        buffers: impl IntoIterator<Item = PipelineBuffers<V, I>>,
        bind_groups: impl IntoIterator<Item = &wgpu::BindGroup>,
    ) where
        V: bytemuck::NoUninit,
        I: bytemuck::NoUninit,
    {
        let depth_view = target.depth_texture().map(|texture| &texture.view);
        self.internal_render(
            &target.texture().view,
            depth_view,
            !target.color_cleared.get(),
            !target.depth_cleared.get(),
            pipeline,
            buffers,
            bind_groups,
        );
        target.color_cleared.set(true);
        if pipeline.descriptor.use_depth && depth_view.is_some() {
            target.depth_cleared.set(true);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn internal_render<V, I>(
        &self,
        target_view: &wgpu::TextureView,
        depth_view: Option<&wgpu::TextureView>,
        clear_color: bool,
        clear_depth: bool,
        pipeline: &Pipeline<V, I>,
        buffers: impl IntoIterator<Item = PipelineBuffers<V, I>>,
        bind_groups: impl IntoIterator<Item = &wgpu::BindGroup>,
    ) where
        V: bytemuck::NoUninit,
        I: bytemuck::NoUninit,
    {
        let mut encoder = self
            .handle
            .device
            .create_command_encoder(&Default::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(pipeline.descriptor.name),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if clear_color {
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            })
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: if let Some(depth_view) = depth_view {
                    pipeline.descriptor.use_depth.then_some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: if clear_depth {
                                    wgpu::LoadOp::Clear(1.0)
                                } else {
                                    wgpu::LoadOp::Load
                                },
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        },
                    )
                } else {
                    None
                },
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for (i, bind_group) in bind_groups.into_iter().enumerate() {
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }

            render_pass.set_pipeline(&pipeline.gpu_pipeline);

            'buffer_loop: for PipelineBuffers {
                vertices,
                instances,
                indices,
            } in buffers
            {
                if let Some(vertex_buffer_slice) = vertices.borrow_buffer() {
                    render_pass.set_vertex_buffer(0, vertex_buffer_slice);

                    let index_count = if let Some(indices) = indices {
                        if let Some(index_buffer_slice) = indices.borrow_buffer() {
                            render_pass
                                .set_index_buffer(index_buffer_slice, wgpu::IndexFormat::Uint32);
                            Some(indices.len())
                        } else {
                            continue 'buffer_loop;
                        }
                    } else {
                        None
                    };

                    let instance_count = if let Some(instances) = instances {
                        if let Some(instance_buffer_slice) = instances.borrow_buffer() {
                            render_pass.set_vertex_buffer(1, instance_buffer_slice);

                            instances.len()
                        } else {
                            continue 'buffer_loop;
                        }
                    } else {
                        render_pass.set_vertex_buffer(1, pipeline.dummy_instance_buffer.slice(..));
                        1
                    };

                    if let Some(index_count) = index_count {
                        render_pass.draw_indexed(
                            0..index_count as u32,
                            0,
                            0..instance_count as u32,
                        );
                    } else {
                        render_pass.draw(0..vertices.len() as u32, 0..instance_count as u32);
                    }
                }
            }
        }

        self.handle.queue.submit(std::iter::once(encoder.finish()));
    }
}
