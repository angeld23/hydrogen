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

impl<D> GuiElement<D> for TextureFrame {
    fn transform(&self) -> GuiTransform {
        self.transform
    }

    fn render(&self, context: &mut GuiContext<D>) -> Vec<GuiPrimitive> {
        let GuiContext { frame, .. } = context;
        let frame = *frame;

        vec![GuiPrimitive {
            absolute_position: self.transform.absolute_position(frame),
            absolute_size: self.transform.absolute_size(frame),
            section: self.section,
            color: self.color,
        }]
    }
}
