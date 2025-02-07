use crate::{builder::GuiBuilder, text::TextLabel, transform::GuiTransform};
use cgmath::{vec2, ElementWise, Vector2};
use hydrogen_core::dependency::{Dependency, DependencyMut};
use hydrogen_data_structures::indexed_container::IndexedContainer;
use hydrogen_graphics::{color::RGBA, texture_provider::TextureProvider, vertex::Vertex2D};
use hydrogen_math::{bbox, rect::OrientedSection};

#[derive(Debug)]
pub struct GuiContext<D> {
    pub frame: Vector2<f32>,
    pub global_frame: Vector2<f32>,
    pub offset: Vector2<f32>,

    pub dependencies: D,
}

impl GuiContext<u8> {
    pub fn new_no_dependencies(frame: Vector2<f32>) -> Self {
        Self {
            frame,
            global_frame: frame,
            offset: vec2(0.0, 0.0),

            dependencies: 0,
        }
    }
}

impl<D> GuiContext<D> {
    pub fn new(frame: Vector2<f32>, dependencies: D) -> Self {
        Self {
            frame,
            global_frame: frame,
            offset: vec2(0.0, 0.0),

            dependencies,
        }
    }

    pub fn builder(self) -> GuiBuilder<D> {
        GuiBuilder::new(self)
    }

    pub fn absolute_position(&self, transform: GuiTransform) -> Vector2<f32> {
        transform.absolute_position(self.frame) + self.offset
    }

    pub fn absolute_size(&self, transform: GuiTransform) -> Vector2<f32> {
        transform.absolute_size(self.frame)
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

    pub fn dep<DD>(&self) -> &DD
    where
        D: Dependency<DD>,
    {
        self.dependencies.dep()
    }

    pub fn dep_mut<DD>(&mut self) -> &mut DD
    where
        D: DependencyMut<DD>,
    {
        self.dependencies.dep_mut()
    }

    pub fn white(&self) -> OrientedSection
    where
        D: Dependency<TextureProvider>,
    {
        self.dependencies.dep().get_section("white")
    }
}

pub trait GuiElement<D> {
    fn transform(&self) -> GuiTransform;
    fn render(&self, context: &mut GuiContext<D>) -> Vec<GuiPrimitive>;
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
