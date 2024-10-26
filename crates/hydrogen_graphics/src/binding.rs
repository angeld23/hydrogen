use crate::{gpu_vec::GpuVec, texture::Texture};

pub type BindGroupFormat = [(wgpu::ShaderStages, wgpu::BindingType)];

pub fn bind_group_format_to_layout_entries(
    format: &BindGroupFormat,
) -> Vec<wgpu::BindGroupLayoutEntry> {
    format
        .iter()
        .copied()
        .enumerate()
        .map(|(i, (stages, binding_type))| wgpu::BindGroupLayoutEntry {
            binding: i as u32,
            visibility: stages,
            ty: binding_type,
            count: None,
        })
        .collect()
}

#[derive(Debug)]
pub struct BindedTexture {
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct BindedBuffer<T>
where
    T: bytemuck::NoUninit,
{
    pub buffer: GpuVec<T>,
    pub bind_group: wgpu::BindGroup,
}
