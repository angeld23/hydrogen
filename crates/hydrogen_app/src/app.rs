use hydrogen_core::{global_dep, global_dependency::set_global_dep};
use hydrogen_graphics::graphics_controller::GraphicsController;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{CursorGrabMode, Window, WindowId},
};

use std::{
    sync::Arc, time::{Duration, Instant}
};

use crate::input::InputController;

mod hydrogen {
    pub use hydrogen_core as core;
}

#[derive(Debug, Clone, Copy)]
pub enum WinitEvent<'a> {
    Window(&'a WindowEvent),
    Device(&'a DeviceEvent),
}

pub trait AppStateHandler {
    #![allow(unused_variables)]

    const TICKS_PER_SECOND: f32 = 20.0;

    fn new(window: Arc<Window>) -> Self;
    /// On frames where a tick occurs, this runs *before* [`AppStateHandler::render`].
    fn tick(&mut self, delta: Duration) {}
    /// - `delta`: The time since the last render call.
    /// - `tick_progress`: A value within `[0, 1)` representing how far we are between the last tick and
    ///   the next tick. This is *always* `0.0` if and only if a tick just occurred.
    fn render(&mut self, delta: Duration, tick_progress: f32) {}
    fn winit_event(&mut self, event: WinitEvent) {}
    fn window_focus_changed(&mut self, focused: bool) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDescriptor {
    pub window_title: String,
}

pub struct App<T>
where
    T: AppStateHandler,
{
    descriptor: AppDescriptor,
    window: Option<Arc<Window>>,
    app_state: Option<T>,
    last_frame: Instant,
    last_tick: Instant,
    next_tick: Instant,
    mouse_locked: bool,
}

impl<T> App<T>
where
    T: AppStateHandler,
{
    pub fn new(descriptor: AppDescriptor) -> Self {
        Self {
            window: None,
            app_state: None,
            last_frame: Instant::now(),
            last_tick: Instant::now(),
            next_tick: Instant::now() + Duration::from_secs_f32(1.0 / T::TICKS_PER_SECOND),
            mouse_locked: false,

            descriptor,
        }
    }
}

impl<T> ApplicationHandler for App<T>
where
    T: AppStateHandler,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes().with_title(&self.descriptor.window_title),
                )
                .unwrap(),
        );
        window.set_ime_allowed(true);

        set_global_dep(GraphicsController::new(Arc::clone(&window)).unwrap(), None);
        set_global_dep(InputController::new(), None);

        let app_state = T::new(Arc::clone(&window));
        self.app_state = Some(app_state);

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let (window, app_state) = match (&self.window, &mut self.app_state) {
            (Some(window), Some(app_state)) => (window, app_state),
            _ => return,
        };

        if window_id != window.id() {
            return;
        }

        global_dep!(mut InputController).winit_event(WinitEvent::Window(&event));
        app_state.winit_event(
            WinitEvent::Window(&event)
        );

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                // device_id,
                // event: input_event,
                // is_synthetic,
                ..
            } => {}
            WindowEvent::RedrawRequested => {
                let frame_time = self.last_frame.elapsed();
                let now = Instant::now();
                self.last_frame = now;

                // tick handling
                if !self.next_tick.elapsed().is_zero() {
                    app_state.tick(self.last_tick.elapsed());

                    //controllers().input_controller.write().tick();
                    
                    global_dep!(mut InputController).tick();

                    self.last_tick = now;
                    self.next_tick += Duration::from_secs_f32(1.0 / T::TICKS_PER_SECOND);
                    if self.next_tick.elapsed() > Duration::from_secs_f32(20.0 / T::TICKS_PER_SECOND) {
                        self.next_tick = now - Duration::from_secs_f32(20.0 / T::TICKS_PER_SECOND)
                    }
                }
                
                let tick_progress = (now - self.last_tick).as_secs_f32() / (self.next_tick - self.last_tick).as_secs_f32();
                // where the magic happens
                app_state.render(frame_time, tick_progress);

                // mouse logic
                let new_mouse_locked = global_dep!(InputController).is_mouse_locked();
                if new_mouse_locked != self.mouse_locked {
                    if new_mouse_locked {
                        window.set_cursor_grab(CursorGrabMode::Locked).unwrap_or_else(|_| {
                            let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                        });
                        window.set_cursor_visible(false);
                    } else {
                        window.set_cursor_grab(CursorGrabMode::None).unwrap();
                        window.set_cursor_visible(true);
                    }
                }
                self.mouse_locked = new_mouse_locked;

                global_dep!(mut InputController).clear_inputs();

                window.request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                global_dep!(mut GraphicsController).resize(new_size);
            }
            WindowEvent::Focused(is_focused) => {
                app_state.window_focus_changed(is_focused);
            }
            _ => {

            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let (_, app_state) = match (&self.window, &mut self.app_state) {
            (Some(window), Some(app_state)) => (window, app_state),
            _ => return,
        };

        global_dep!(mut InputController)
            .winit_event(WinitEvent::Device(&event));
        app_state.winit_event(
            WinitEvent::Device(&event)
        );
    }
}
