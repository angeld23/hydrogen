use std::cell::Cell;

use cgmath::{ElementWise, Vector2, vec2};
use hydrogen_core::global_dep;
use hydrogen_data_structures::indexed_container::IndexedContainer;
use hydrogen_graphics::{color::RGBA, texture_provider::TextureProvider, vertex::Vertex2D};
use hydrogen_math::{bbox, rect::OrientedSection};

use crate::{builder::GuiBuilder, text::TextLabel, transform::GuiTransform};

mod hydrogen {
    pub use hydrogen_core as core;
}

#[derive(Debug, Clone)]
pub struct GuiContext {
    pub frame: Cell<Vector2<f32>>,
    pub global_frame: Cell<Vector2<f32>>,
    pub offset: Cell<Vector2<f32>>,
}

impl GuiContext {
    pub fn new(frame: Vector2<f32>) -> Self {
        Self {
            frame: frame.into(),
            global_frame: frame.into(),
            offset: vec2(0.0, 0.0).into(),
        }
    }

    pub fn frame(&self) -> Vector2<f32> {
        self.frame.get()
    }

    pub fn global_frame(&self) -> Vector2<f32> {
        self.global_frame.get()
    }

    pub fn offset(&self) -> Vector2<f32> {
        self.offset.get()
    }

    pub fn builder(self) -> GuiBuilder {
        GuiBuilder::new(self)
    }

    pub fn absolute_position(&self, transform: GuiTransform) -> Vector2<f32> {
        transform.absolute_position(self.frame()) + self.offset()
    }

    pub fn absolute_size(&self, transform: GuiTransform) -> Vector2<f32> {
        transform.absolute_size(self.frame())
    }

    /// (absolute_position, absolute_size)
    pub fn absolute(&self, transform: GuiTransform) -> (Vector2<f32>, Vector2<f32>) {
        (
            self.absolute_position(transform),
            self.absolute_size(transform),
        )
    }

    pub fn char_pixel_height(&self, transform: GuiTransform, lines: u32) -> f32 {
        TextLabel::get_max_char_pixel_height(self.absolute_size(transform).y, lines)
    }

    pub fn white(&self) -> OrientedSection {
        global_dep!(TextureProvider).get_section("white")
    }
}

pub trait GuiElement {
    fn transform(&self) -> GuiTransform;
    fn render(&self, context: &GuiContext) -> Vec<GuiPrimitive>;
}

#[derive(Debug, Clone, Copy)]
pub struct GuiPrimitive {
    pub absolute_position: Vector2<f32>,
    pub absolute_size: Vector2<f32>,
    pub section: OrientedSection,
    pub color: RGBA,
}

impl GuiPrimitive {
    pub fn vertices(&self, frame: Vector2<f32>) -> IndexedContainer<Vertex2D> {
        if !self.color.is_visible() {
            return IndexedContainer::default();
        }

        let corner_0 = self.absolute_position.div_element_wise(frame);
        let corner_1 = corner_0 + self.absolute_size.div_element_wise(frame);
        let rect = bbox!(corner_0, corner_1);

        let color = [self.color.r, self.color.g, self.color.b, self.color.a];

        let uv = self.section.uv_corners();
        let tex_index = self.section.section.layer_index;

        IndexedContainer {
            items: vec![
                Vertex2D {
                    pos: rect.get_corner([false, false]),
                    uv: uv.top_left,
                    tex_index,
                    color,
                },
                Vertex2D {
                    pos: rect.get_corner([false, true]),
                    uv: uv.bottom_left,
                    tex_index,
                    color,
                },
                Vertex2D {
                    pos: rect.get_corner([true, true]),
                    uv: uv.bottom_right,
                    tex_index,
                    color,
                },
                Vertex2D {
                    pos: rect.get_corner([true, false]),
                    uv: uv.top_right,
                    tex_index,
                    color,
                },
            ],
            indices: vec![0, 1, 2, 2, 3, 0],
        }
    }
}
