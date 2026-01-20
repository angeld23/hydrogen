use cgmath::{Vector2, vec2};
use derive_more::*;
use hydrogen_math::bounding_box::BBox2;
use linear_map::LinearMap;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{Key, NamedKey, SmolStr},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
};

use crate::app::WinitEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into)]
pub struct GuiComponentId(pub u128);

impl Default for GuiComponentId {
    fn default() -> Self {
        Self::generate()
    }
}

impl GuiComponentId {
    pub fn generate() -> Self {
        Self(rand::random())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum Input {
    CharacterKey(SmolStr),
    NamedKey(NamedKey),
    MouseButton(MouseButton),
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Self::CharacterKey(value.into())
    }
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Self::CharacterKey(value.into())
    }
}

impl From<&String> for Input {
    fn from(value: &String) -> Self {
        Self::CharacterKey(value.into())
    }
}

#[derive(Debug)]
pub struct InputController {
    // (input -> was_last_frame)
    held_inputs: LinearMap<Input, bool>,
    pressed_inputs: LinearMap<Input, bool>,
    pressed_or_repeated_inputs: LinearMap<Input, bool>,
    released_inputs: LinearMap<Input, bool>,

    mouse_delta: Vector2<f32>,
    scroll_delta: f32,
    cursor_position: Vector2<f32>,
    cursor_in_window: bool,

    just_typed: String,
    just_typed_this_tick: String,
    focused_component_id: Option<GuiComponentId>,
    contested_hover: Option<(GuiComponentId, BBox2)>,
    hovered_component_id: Option<GuiComponentId>,
    in_a_menu_next_frame: bool,
    in_a_menu: bool,
}

impl Default for InputController {
    fn default() -> Self {
        Self {
            held_inputs: Default::default(),
            pressed_inputs: Default::default(),
            released_inputs: Default::default(),
            pressed_or_repeated_inputs: Default::default(),

            mouse_delta: vec2(0.0, 0.0),
            scroll_delta: 0.0,
            cursor_position: vec2(0.0, 0.0),
            cursor_in_window: false,

            just_typed: Default::default(),
            just_typed_this_tick: Default::default(),
            focused_component_id: None,
            contested_hover: None,
            hovered_component_id: None,
            in_a_menu_next_frame: false,
            in_a_menu: false,
        }
    }
}

macro_rules! input_is {
    ($fn_name:ident, $tick_fn_name:ident, $map:ident) => {
        pub fn $fn_name(&self, input: impl Into<Input>) -> bool {
            self.$map.get(&input.into()) == Some(&true)
        }

        pub fn $tick_fn_name(&self, input: impl Into<Input>) -> bool {
            self.$map.contains_key(&input.into())
        }
    };
}

macro_rules! consume {
    ($fn_name:ident, $tick_fn_name:ident, $map:ident) => {
        pub fn $fn_name(&mut self, input: impl Into<Input>) -> bool {
            self.$map.remove(&input.into()) == Some(true)
        }

        pub fn $tick_fn_name(&mut self, input: impl Into<Input>) -> bool {
            self.$map.remove(&input.into()).is_some()
        }
    };
}

macro_rules! get_all {
    ($fn_name:ident, $tick_fn_name:ident, $map:ident) => {
        pub fn $fn_name(&mut self) -> Vec<Input> {
            self.$map
                .iter()
                .filter_map(|(input, &was_last_frame)| was_last_frame.then_some(input.clone()))
                .collect()
        }

        pub fn $tick_fn_name(&self) -> Vec<Input> {
            self.$map.keys().cloned().collect()
        }
    };
}

impl InputController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_mouse_locked(&self) -> bool {
        self.focused_component_id.is_none() && !self.in_a_menu
    }

    input_is!(held, held_tick, held_inputs);
    input_is!(pressed, pressed_tick, pressed_inputs);
    input_is!(
        pressed_or_repeated,
        pressed_or_repeated_tick,
        pressed_or_repeated_inputs
    );
    input_is!(released, released_tick, released_inputs);

    consume!(consume_held, consume_held_tick, held_inputs);
    consume!(consume_pressed, consume_pressed_tick, pressed_inputs);
    consume!(
        consume_pressed_or_released,
        consume_pressed_or_released_tick,
        pressed_or_repeated_inputs
    );
    consume!(consume_released, consume_released_tick, released_inputs);

    pub fn consume_input(&mut self, input: impl Into<Input>) -> bool {
        let input = input.into();

        let mut consumed = false;
        consumed |= self.consume_held(input.clone());
        consumed |= self.consume_pressed(input.clone());
        consumed |= self.consume_pressed_or_released(input.clone());
        consumed |= self.consume_released(input);

        consumed
    }

    pub fn consume_input_tick(&mut self, input: impl Into<Input>) -> bool {
        let input = input.into();

        let mut consumed = false;
        consumed |= self.consume_held_tick(input.clone());
        consumed |= self.consume_pressed_tick(input.clone());
        consumed |= self.consume_pressed_or_released_tick(input.clone());
        consumed |= self.consume_released_tick(input);

        consumed
    }

    get_all!(all_held, all_held_tick, held_inputs);
    get_all!(all_pressed, all_pressed_tick, pressed_inputs);
    get_all!(
        all_pressed_or_repeated,
        all_pressed_or_repeated_tick,
        pressed_or_repeated_inputs
    );
    get_all!(all_released, all_released_tick, released_inputs);

    /// Only valid if mouse is locked
    pub fn mouse_delta(&self) -> Vector2<f32> {
        self.mouse_delta
    }

    pub fn cursor_position(&self) -> Vector2<f32> {
        self.cursor_position
    }

    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    pub fn just_typed(&self) -> &str {
        &self.just_typed
    }

    pub fn just_typed_tick(&self) -> &str {
        &self.just_typed_this_tick
    }

    pub fn emulate_just_typed(&mut self, text: &str) {
        self.just_typed.push_str(text);
        self.just_typed_this_tick.push_str(text);
    }

    pub fn tick(&mut self) {
        self.just_typed_this_tick.clear();

        for map in [
            &mut self.held_inputs,
            &mut self.pressed_inputs,
            &mut self.pressed_or_repeated_inputs,
            &mut self.released_inputs,
        ] {
            let keys_to_remove: Vec<Input> = map
                .iter()
                .filter_map(|(input, was_last_frame)| {
                    if !was_last_frame {
                        Some(input.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for key in keys_to_remove {
                map.remove(&key);
            }
        }
    }

    pub fn clear_inputs(&mut self) {
        self.mouse_delta = vec2(0.0, 0.0);
        self.scroll_delta = 0.0;

        for map in [
            //&mut self.held_inputs,
            &mut self.pressed_inputs,
            &mut self.pressed_or_repeated_inputs,
            &mut self.released_inputs,
        ] {
            for (_, was_last_frame) in map.iter_mut() {
                *was_last_frame = false;
            }
        }

        self.just_typed.clear();

        self.hovered_component_id = self.contested_hover.take().map(|(id, _)| id);
        self.in_a_menu = self.in_a_menu_next_frame;
        self.in_a_menu_next_frame = false;
    }

    pub fn focused_component_id(&self) -> Option<GuiComponentId> {
        self.focused_component_id
    }

    pub fn component_is_focused(&self, id: GuiComponentId) -> bool {
        self.focused_component_id == Some(id)
    }

    pub fn unfocus(&mut self) -> Option<GuiComponentId> {
        self.focused_component_id.take()
    }

    pub fn unfocus_component(&mut self, id: GuiComponentId) -> bool {
        if self.focused_component_id == Some(id) {
            self.focused_component_id = None;
            true
        } else {
            false
        }
    }

    pub fn in_a_menu(&self) -> bool {
        self.in_a_menu
    }

    pub fn set_focus(&mut self, id: GuiComponentId) -> Option<GuiComponentId> {
        self.focused_component_id.replace(id)
    }

    pub fn try_set_focus(&mut self, id: GuiComponentId) -> bool {
        let uncontested = self.focused_component_id.is_none();
        if uncontested {
            self.set_focus(id);
        }
        uncontested
    }

    pub fn contest_mouse_hover(&mut self, id: GuiComponentId, bounding_box: BBox2) {
        if !self.cursor_in_window || self.is_mouse_locked() {
            return;
        }
        if !bounding_box.point_is_within(self.cursor_position) {
            return;
        }

        self.contested_hover = Some((id, bounding_box));
    }

    pub fn component_is_hovered(&self, id: GuiComponentId) -> bool {
        self.hovered_component_id == Some(id)
    }

    pub fn report_in_a_menu(&mut self) {
        self.in_a_menu_next_frame = true;
    }

    pub fn is_movement_suppressed(&self) -> bool {
        self.focused_component_id.is_some() || !self.is_mouse_locked()
    }

    pub fn winit_event(&mut self, winit_event: WinitEvent) {
        match winit_event {
            WinitEvent::Window(event) => match event {
                WindowEvent::KeyboardInput { event, .. } => {
                    if self.cursor_in_window
                        && let Some(ref text) = event.text
                    {
                        for character in text.chars() {
                            self.just_typed.push(character);
                        }
                    }

                    let key = event.key_without_modifiers();
                    let pressed = event.state.is_pressed();

                    let input = match key {
                        Key::Character(character) => Input::CharacterKey(character),
                        Key::Named(named_key) => Input::NamedKey(named_key),
                        _ => return,
                    };

                    if pressed {
                        if !self.cursor_in_window {
                            return;
                        }

                        if !event.repeat {
                            self.held_inputs.insert(input.clone(), true);
                            self.pressed_inputs.insert(input.clone(), true);
                        }
                        self.pressed_or_repeated_inputs.insert(input, true);
                    } else {
                        if self.held_inputs.get(&input).is_some() {
                            self.held_inputs.insert(input.clone(), false);
                        }
                        self.released_inputs.insert(input, true);
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if state.is_pressed() {
                        if !self.cursor_in_window {
                            return;
                        }
                        self.held_inputs.insert((*button).into(), true);
                        self.pressed_inputs.insert((*button).into(), true);
                        self.pressed_or_repeated_inputs
                            .insert((*button).into(), true);
                    } else {
                        if self.held_inputs.get(&(*button).into()).is_some() {
                            self.held_inputs.insert((*button).into(), false);
                        }
                        self.released_inputs.insert((*button).into(), true);
                    };
                }
                WindowEvent::CursorEntered { .. } => {
                    self.cursor_in_window = true;
                }
                WindowEvent::CursorLeft { .. } => {
                    self.cursor_in_window = false;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.cursor_position = vec2(position.x as f32, position.y as f32);
                }
                WindowEvent::Ime(Ime::Commit(text)) => {
                    if self.cursor_in_window {
                        self.just_typed.push_str(text);
                    }
                }
                _ => {}
            },
            WinitEvent::Device(event) => match event {
                DeviceEvent::MouseWheel { delta } if self.cursor_in_window => {
                    self.scroll_delta += match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => {
                            *y as f32 / 16.0
                        }
                    }
                }
                DeviceEvent::MouseMotion { delta } if self.is_mouse_locked() => {
                    self.mouse_delta += vec2(delta.0 as f32, delta.1 as f32)
                }
                _ => {}
            },
        }
    }
}
