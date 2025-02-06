use std::collections::BTreeMap;

use crate::{
    binding::BindedTexture,
    gpu_handle::GpuHandle,
    texture::{self, Texture},
};
use hydrogen_math::{
    rect::{OrientedSection, PackedSection},
    rect_packer::{PackResult, RectPacker},
};

#[derive(Debug)]
pub struct TextureProvider {
    main_texture: BindedTexture,
    texture_sections: BTreeMap<String, PackedSection>,
    reserved_textures: BTreeMap<String, wgpu::Texture>,
    packer: RectPacker,
    handle: GpuHandle,
}

impl TextureProvider {
    pub const TEXTURE_SIDE_LENGTH: u32 = 2048;
    pub const PADDING: u32 = 2;

    fn texture_descriptor(layers: u32) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            size: wgpu::Extent3d {
                width: Self::TEXTURE_SIDE_LENGTH,
                height: Self::TEXTURE_SIDE_LENGTH,
                // we need at least 2 layers, otherwise a texture view created with a
                // default descriptor (like in Texture::new) will have a dimension of D2 instead of D2Array
                depth_or_array_layers: layers.max(2),
            },
            ..*texture::TEXTURE_IMAGE
        }
    }

    pub fn new(handle: &GpuHandle) -> Self {
        Self {
            main_texture: handle.binded_texture(
                &handle.create_bind_group_layout(Texture::ARRAY_BIND_GROUP_LAYOUT),
                Texture::new(
                    handle,
                    &Self::texture_descriptor(1),
                    &texture::SAMPLER_PIXELATED,
                ),
            ),
            texture_sections: Default::default(),
            reserved_textures: Default::default(),
            packer: RectPacker::new(
                Self::TEXTURE_SIDE_LENGTH,
                Self::TEXTURE_SIDE_LENGTH,
                Self::PADDING,
            ),
            handle: handle.clone(),
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.main_texture.bind_group
    }

    pub fn layer_count(&self) -> u32 {
        self.main_texture
            .texture
            .inner_texture
            .depth_or_array_layers()
    }

    pub fn reserve_slot(&mut self, name: impl Into<String>, width: u32, height: u32) -> bool {
        self.packer.reserve(name, width, height)
    }

    pub fn reserve_texture(
        &mut self,
        name: impl Into<String>,
        texture: wgpu::Texture,
    ) -> Option<wgpu::Texture> {
        let name = name.into();
        if !self
            .packer
            .reserve(&name, texture.width(), texture.height())
        {
            Some(texture)
        } else {
            self.reserved_textures.insert(name, texture);
            None
        }
    }

    pub fn reset_main_texture(&mut self, layers: u32) {
        self.main_texture = self.handle.binded_texture(
            &self
                .handle
                .create_bind_group_layout(Texture::ARRAY_BIND_GROUP_LAYOUT),
            Texture::new(
                &self.handle,
                &Self::texture_descriptor(layers),
                &texture::SAMPLER_PIXELATED,
            ),
        );
    }

    pub fn pack(&mut self) {
        let packer = std::mem::replace(
            &mut self.packer,
            RectPacker::new(
                Self::TEXTURE_SIDE_LENGTH,
                Self::TEXTURE_SIDE_LENGTH,
                Self::PADDING,
            ),
        );
        let PackResult {
            total_layers,
            sections,
        } = packer.pack();

        if !sections.contains_key("fallback") {
            println!("[H1 | WARN] Texture provider packed with no 'fallback' texture! ANY attempt to fetch a non-existent texture will result in a panic.");
        }
        if !sections.contains_key("font") {
            println!(
                "[H1 | WARN] Texture provider packed with no 'font' texture! Text cannot be displayed."
            );
        }

        self.reset_main_texture(total_layers);
        self.texture_sections = sections;

        for (name, texture) in std::mem::take(&mut self.reserved_textures) {
            self.write_texture(name, &texture);
        }
    }

    pub fn write_texture(&self, name: impl Into<String>, texture: &wgpu::Texture) -> bool {
        let name = name.into();
        if let Some(&section) = self.texture_sections.get(&name) {
            if section.layer_index < self.layer_count() {
                let mut encoder = self
                    .handle
                    .device
                    .create_command_encoder(&Default::default());

                encoder.copy_texture_to_texture(
                    texture.as_image_copy(),
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.main_texture.texture.inner_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: (section.uv.min()[0] * Self::TEXTURE_SIDE_LENGTH as f32) as u32,
                            y: (section.uv.min()[1] * Self::TEXTURE_SIDE_LENGTH as f32) as u32,
                            z: section.layer_index,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    texture.size(),
                );

                self.handle.queue.submit(std::iter::once(encoder.finish()));

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_packed_section(&self, name: &str) -> PackedSection {
        *self
            .texture_sections
            .get(name)
            .unwrap_or_else(|| self.texture_sections.get("fallback").unwrap())
    }

    pub fn get_section(&self, name: &str) -> OrientedSection {
        self.get_packed_section(name).unoriented()
    }
}
