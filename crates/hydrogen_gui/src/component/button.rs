use hydrogen_core::{
    dependency::DependencyMut,
    input::{GuiComponentId, InputController},
};
use hydrogen_math::bbox;
use winit::event::MouseButton;

use crate::{element::GuiContext, transform::GuiTransform};

#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    id: GuiComponentId,

    hovering: bool,
    left_held: bool,
    right_held: bool,

    last_hovering: bool,
    last_left_held: bool,
    last_right_held: bool,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    pub fn new() -> Self {
        Self {
            id: GuiComponentId::generate(),

            hovering: false,
            left_held: false,
            right_held: false,

            last_hovering: false,
            last_left_held: false,
            last_right_held: false,
        }
    }

    pub fn update<D>(&mut self, context: &mut GuiContext<D>, transform: GuiTransform)
    where
        D: DependencyMut<InputController>,
    {
        let (absolute_position, absolute_size) = context.absolute(transform);
        let bounding_box = bbox!(absolute_position, absolute_position + absolute_size);

        let input_controller = context.dep_mut();

        // contest for next frame
        input_controller.contest_mouse_hover(self.id, bounding_box);

        let hovered = input_controller.component_is_hovered(self.id);
        let left_held = hovered
            && if self.left_held {
                input_controller.held(MouseButton::Left)
            } else {
                input_controller.pressed(MouseButton::Left)
            };
        let right_held = hovered
            && if self.right_held {
                input_controller.held(MouseButton::Right)
            } else {
                input_controller.pressed(MouseButton::Right)
            };

        self.last_left_held = self.left_held;
        self.last_right_held = self.right_held;
        self.last_hovering = self.hovering;

        self.left_held = left_held;
        self.right_held = right_held;
        self.hovering = hovered;
    }

    pub fn reset(&mut self) {
        self.hovering = false;
        self.left_held = false;
        self.right_held = false;

        self.last_hovering = false;
        self.last_left_held = false;
        self.last_right_held = false;
    }

    pub fn hovering(&self) -> bool {
        self.hovering
    }

    pub fn hover_started(&self) -> bool {
        self.hovering && (self.hovering != self.last_hovering)
    }

    pub fn hover_ended(&self) -> bool {
        !self.hovering && (self.hovering != self.last_hovering)
    }

    pub fn left_held(&self) -> bool {
        self.left_held
    }

    pub fn left_pressed(&self) -> bool {
        self.left_held && (self.left_held != self.last_left_held)
    }

    pub fn left_released(&self) -> bool {
        !self.left_held && (self.left_held != self.last_left_held)
    }

    pub fn right_held(&self) -> bool {
        self.right_held
    }

    pub fn right_pressed(&self) -> bool {
        self.right_held && (self.right_held != self.last_right_held)
    }

    pub fn right_released(&self) -> bool {
        !self.right_held && (self.right_held != self.last_right_held)
    }
}
