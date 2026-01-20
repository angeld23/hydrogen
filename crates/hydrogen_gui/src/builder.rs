use std::sync::Mutex;

use hydrogen_data_structures::indexed_container::IndexedContainer;
use hydrogen_graphics::vertex::Vertex2D;

use crate::element::{GuiContext, GuiElement};

#[derive(Debug)]
pub struct GuiBuilder<D> {
    pub context: GuiContext<D>,
    vertices: Mutex<IndexedContainer<Vertex2D>>,
}

impl<D> GuiBuilder<D> {
    pub fn new(context: GuiContext<D>) -> Self {
        Self {
            context,
            vertices: Mutex::new(Default::default()),
        }
    }

    pub fn element(&self, element: impl GuiElement<D>) -> &Self {
        let primitives = element.render(&self.context);
        let mut vertices = self.vertices.lock().unwrap();

        vertices.items.reserve(primitives.len() * 4);
        vertices.indices.reserve(primitives.len() * 6);
        for mut primitive in primitives {
            primitive.absolute_position += self.context.offset();
            vertices.push_container(primitive.vertices(self.context.frame()));
        }
        self
    }

    pub fn element_children(
        &self,
        element: impl GuiElement<D>,
        children: impl FnOnce(&Self),
    ) -> &Self {
        let context = &self.context;

        let old_frame = context.frame();
        let old_offset = context.offset();

        let transform = element.transform();

        self.element(element);

        context.frame.set(transform.absolute_size(old_frame));
        context
            .offset
            .set(old_offset + transform.absolute_position(old_frame));

        children(self);

        context.frame.set(old_frame);
        context.offset.set(old_offset);

        self
    }

    pub fn finish(self) -> IndexedContainer<Vertex2D> {
        self.vertices.into_inner().unwrap()
    }
}
