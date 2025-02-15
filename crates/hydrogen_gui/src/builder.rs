use std::{cell::UnsafeCell, sync::Mutex};

use hydrogen_data_structures::indexed_container::IndexedContainer;
use hydrogen_graphics::vertex::Vertex2D;

use crate::element::{GuiContext, GuiElement};

#[derive(Debug)]
pub struct GuiBuilder<D> {
    vertices: Mutex<IndexedContainer<Vertex2D>>,
    context: UnsafeCell<GuiContext<D>>,
}

impl<D> GuiBuilder<D> {
    pub fn new(context: GuiContext<D>) -> Self {
        Self {
            vertices: Mutex::new(Default::default()),
            context: UnsafeCell::new(context),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn context(&self) -> &mut GuiContext<D> {
        // this is awful. don't do things like this
        unsafe { self.context.as_mut_unchecked() }
    }

    pub fn element(&self, element: impl GuiElement<D>) -> &Self {
        let primitives = element.render(self.context());
        let context = self.context();
        let mut vertices = self.vertices.lock().unwrap();

        vertices.items.reserve(primitives.len() * 4);
        vertices.indices.reserve(primitives.len() * 6);
        for mut primitive in primitives {
            primitive.absolute_position += context.offset;
            vertices.push_container(primitive.vertices(context.frame));
        }
        self
    }

    pub fn element_children(
        &self,
        element: impl GuiElement<D>,
        children: impl FnOnce(&Self),
    ) -> &Self {
        let context = self.context();

        let old_frame = context.frame;
        let old_offset = context.offset;

        let transform = element.transform();

        self.element(element);

        context.frame = transform.absolute_size(old_frame);
        context.offset = old_offset + transform.absolute_position(old_frame);

        children(self);

        context.frame = old_frame;
        context.offset = old_offset;

        self
    }

    pub fn finish(self) -> IndexedContainer<Vertex2D> {
        self.vertices.into_inner().unwrap()
    }
}
