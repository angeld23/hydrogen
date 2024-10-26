use cgmath::{vec2, Vector2};

use crate::{gpu_handle::GpuHandle, texture::Texture};
use std::cell::Cell;

#[derive(Debug)]
pub struct RenderTarget {
    pub(crate) texture: Texture,
    pub(crate) color_cleared: Cell<bool>,

    pub(crate) depth_texture: Option<Texture>,
    pub(crate) depth_cleared: Cell<bool>,
}

impl RenderTarget {
    pub fn new(handle: &GpuHandle, texture: Texture) -> Self {
        Self {
            depth_texture: Some(Texture::create_depth_texture(
                handle,
                texture.inner_texture.width(),
                texture.inner_texture.height(),
            )),
            texture,
            color_cleared: Cell::new(false),
            depth_cleared: Cell::new(false),
        }
    }

    pub fn no_depth(texture: Texture) -> Self {
        Self {
            texture,
            color_cleared: Cell::new(false),
            depth_texture: None,
            depth_cleared: Cell::new(false),
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn width(&self) -> u32 {
        self.texture.inner_texture.width()
    }

    pub fn height(&self) -> u32 {
        self.texture.inner_texture.height()
    }

    pub fn frame(&self) -> Vector2<f32> {
        vec2(self.width() as f32, self.height() as f32)
    }

    /// width / height
    pub fn aspect_ratio(&self) -> f32 {
        self.width() as f32 / self.height() as f32
    }

    pub fn depth_texture(&self) -> Option<&Texture> {
        self.depth_texture.as_ref()
    }

    pub fn clear_color(&self) {
        self.color_cleared.set(false);
    }

    pub fn clear_depth(&self) {
        self.depth_cleared.set(false);
    }

    pub fn clear(&self) {
        self.clear_color();
        self.clear_depth();
    }
}
