use crate::{
    element::{GuiContext, GuiElement, GuiPrimitive},
    transform::GuiTransform,
};
use hydrogen_graphics::color::RGBA;
use hydrogen_math::rect::OrientedSection;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureFrame {
    pub transform: GuiTransform,
    pub color: RGBA,
    pub section: OrientedSection,
}

impl GuiElement for TextureFrame {
    fn transform(&self) -> GuiTransform {
        self.transform
    }

    fn render(&self, context: &GuiContext) -> Vec<GuiPrimitive> {
        let frame = context.frame();

        vec![GuiPrimitive {
            absolute_position: self.transform.absolute_position(frame),
            absolute_size: self.transform.absolute_size(frame),
            section: self.section,
            color: self.color,
        }]
    }
}
