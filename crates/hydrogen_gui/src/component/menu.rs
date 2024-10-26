use hydrogen_graphics::color::RGBA;

pub const COLOR_BUTTON_DEFAULT: RGBA = RGBA::rgb(1.0 / 24.0, 1.0 / 24.0, 1.0 / 24.0);
pub const LIST_MARGIN_PORTION: f32 = 0.01;
pub const OUTLINE_THICKNESS_PORTION: f32 = 0.0025;

pub fn get_outline_thickness(screen_height: f32) -> f32 {
    (OUTLINE_THICKNESS_PORTION * screen_height).ceil()
}

pub fn get_list_margin(screen_height: f32) -> f32 {
    (LIST_MARGIN_PORTION * screen_height).ceil()
}
