use crate::{
    binding::{bind_group_format_to_layout_entries, BindGroupFormat, BindedBuffer, BindedTexture},
    gpu_vec::GpuVec,
    texture::Texture,
};
use futures::{channel::oneshot, executor};
use image::RgbaImage;

/// A handle to both a [wgpu::Device] and a [wgpu::Queue].
///
/// Contains useful wrapper functions for various operations performed using the device and queue.
#[derive(Debug, Clone)]
pub struct GpuHandle {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuHandle {
    pub fn create_bind_group_layout(&self, format: &BindGroupFormat) -> wgpu::BindGroupLayout {
        self.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &bind_group_format_to_layout_entries(format),
            })
    }

    pub fn create_bind_group(
        &self,
        layout: &wgpu::BindGroupLayout,
        resources: Vec<wgpu::BindingResource>,
    ) -> wgpu::BindGroup {
        let entries: Vec<wgpu::BindGroupEntry<'_>> = resources
            .into_iter()
            .enumerate()
            .map(|(index, binding_resource)| wgpu::BindGroupEntry {
                binding: index as u32,
                resource: binding_resource,
            })
            .collect();

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        })
    }

    pub fn binded_texture(
        &self,
        layout: &wgpu::BindGroupLayout,
        texture: Texture,
    ) -> BindedTexture {
        let bind_group = self.create_bind_group(
            layout,
            vec![
                wgpu::BindingResource::TextureView(&texture.view),
                wgpu::BindingResource::Sampler(&texture.sampler),
            ],
        );
        BindedTexture {
            texture,
            bind_group,
        }
    }

    pub fn binded_buffer<T>(
        &self,
        layout: &wgpu::BindGroupLayout,
        buffer: GpuVec<T>,
    ) -> BindedBuffer<T>
    where
        T: bytemuck::NoUninit,
    {
        let bind_group = self.create_bind_group(layout, vec![buffer.buffer().as_entire_binding()]);
        BindedBuffer { buffer, bind_group }
    }

    pub fn read_buffer(&self, buffer: &wgpu::Buffer) -> Vec<u8> {
        let data = {
            let buffer_slice = buffer.slice(..);
            let (tx, rx) = oneshot::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            self.device
                .poll(wgpu::PollType::wait_indefinitely())
                .unwrap();
            executor::block_on(rx).unwrap().unwrap();

            let view = buffer_slice.get_mapped_range();
            view.to_vec()
        };
        buffer.unmap();

        data
    }

    pub fn read_texture(&self, texture: &wgpu::Texture) -> Vec<u8> {
        assert!(
            (texture.size().width * 4).is_multiple_of(256),
            "Texture row size must a be multiple of 256"
        );

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let size = texture.size();
        let buffer_length = (size.width * size.height * 4) as wgpu::BufferAddress;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_length,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * 4),
                    rows_per_image: None,
                },
            },
            size,
        );
        self.queue.submit(std::iter::once(encoder.finish()));

        self.read_buffer(&buffer)
    }

    pub fn read_texture_to_image(&self, texture: &wgpu::Texture) -> RgbaImage {
        let image_bytes = self.read_texture(texture);
        RgbaImage::from_raw(texture.width(), texture.height(), image_bytes).unwrap()
    }
}
