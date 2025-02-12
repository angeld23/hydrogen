use super::{
    button::Button,
    menu::{get_list_margin, get_outline_thickness, COLOR_BUTTON_DEFAULT},
};
use crate::{
    builder::GuiBuilder,
    text::{StyledText, TextBackgroundType, TextLabel},
    texture_frame::TextureFrame,
    transform::GuiTransform,
};
use cgmath::vec2;
use hydrogen_core::{
    dependency::{Dependency, DependencyMut},
    input::InputController,
};
use hydrogen_graphics::{color::RGBA, texture_provider::TextureProvider};

#[derive(Debug)]
pub struct TextButton {
    pub button: Button,
    pub text: StyledText,
    pub color: RGBA,
}

impl Default for TextButton {
    fn default() -> Self {
        Self {
            button: Default::default(),
            text: Default::default(),
            color: COLOR_BUTTON_DEFAULT,
        }
    }
}

impl TextButton {
    pub fn render<D>(&mut self, builder: &GuiBuilder<D>, text_label: TextLabel)
    where
        D: Dependency<TextureProvider> + DependencyMut<InputController>,
    {
        let context = &mut builder.context();

        self.button.update(context, text_label.transform);

        let outline_thickness = get_outline_thickness(context.global_frame.y);

        let (absolute_position, absolute_size) = context.absolute(text_label.transform);

        builder.element(TextureFrame {
            transform: text_label.transform,
            color: if self.button.hovering() {
                RGBA::WHITE
            } else {
                RGBA::BLACK
            },
            section: context.white(),
        });

        builder.element(TextLabel {
            transform: GuiTransform::from_absolute(
                absolute_position + vec2(outline_thickness, outline_thickness),
                absolute_size - vec2(outline_thickness, outline_thickness) * 2.0,
            ),
            text: self.text.clone(),
            background_color: COLOR_BUTTON_DEFAULT,
            background_type: TextBackgroundType::Full,
            ..text_label
        });
    }
}

pub fn button_list<D>(
    builder: &GuiBuilder<D>,
    container: GuiTransform,
    button_rows: &mut [&mut [&mut TextButton]],
    render_buttons: bool,
) where
    D: Dependency<TextureProvider> + DependencyMut<InputController>,
{
    if button_rows.is_empty() {
        return;
    }

    let context = &mut builder.context();

    let row_count = button_rows.len();
    let pixel_margin = get_list_margin(context.global_frame.y);

    let (absolute_position, absolute_size) = context.absolute(container);
    // the whole frame *minus* the total margin, divided by the amount of rows
    let button_pixel_height =
        (absolute_size.y - (row_count - 1) as f32 * pixel_margin) / row_count as f32;
    let char_pixel_height = (button_pixel_height / 2.0).floor();

    for (row_number, buttons) in button_rows.iter_mut().enumerate() {
        if buttons.is_empty() {
            continue;
        }

        let button_count = buttons.len();

        let pixel_y_offset = (button_pixel_height + pixel_margin) * row_number as f32;
        // same kind of thing as button_pixel_height
        let button_pixel_width =
            (absolute_size.x - (button_count - 1) as f32 * pixel_margin) / button_count as f32;
        for (button_number, button) in buttons.iter_mut().enumerate() {
            let pixel_x_offset = (button_pixel_width + pixel_margin) * button_number as f32;
            let transform = GuiTransform::from_absolute(
                absolute_position + vec2(pixel_x_offset, pixel_y_offset),
                vec2(button_pixel_width, button_pixel_height),
            );

            if !render_buttons {
                button.button.reset();
            } else {
                button.render(
                    builder,
                    TextLabel {
                        transform,
                        char_pixel_height,
                        text_alignment: TextLabel::ALIGN_MIDDLE_CENTER,
                        ..Default::default()
                    },
                );
            }
        }
    }
}

#[macro_export]
macro_rules! tb {
    ($text:expr) => {
        TextButton {
            text: StyledText::single_section(
                $text,
                TextStyling {
                    text_color: GuiColor::WHITE,
                    drop_shadow_color: GuiColor::INVISIBLE,
                    bold: false,
                },
            ),
            ..Default::default()
        }
    };
}

pub use tb;
