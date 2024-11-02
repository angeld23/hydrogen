use crate::input::InputController;
use hydrogen_core_proc_macro::DependencyProvider;
use hydrogen_graphics::graphics_controller::GraphicsController;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{CursorGrabMode, Window, WindowId},
};

mod hydrogen {
    pub use crate as core;
}

#[derive(Debug, Clone, Copy)]
pub enum WinitEvent<'a> {
    Window(&'a WindowEvent),
    Device(&'a DeviceEvent),
}

pub trait AppStateHandler {
    #![allow(unused_variables)]

    const TICKS_PER_SECOND: f32 = 20.0;

    fn new(window: Arc<Window>, controllers: &mut Controllers) -> Self;
    fn render(&mut self, delta: Duration, controllers: &mut Controllers) {}
    fn tick(&mut self, delta: Duration, controllers: &mut Controllers) {}
    fn winit_event(&mut self, event: WinitEvent, controllers: &mut Controllers) {}
    fn window_focus_changed(&mut self, focused: bool, controllers: &mut Controllers) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDescriptor {
    pub window_title: String,
}

#[derive(Debug, DependencyProvider)]
pub struct Controllers {
    #[dep]
    #[dep_mut]
    pub graphics_controller: GraphicsController,
    #[dep]
    #[dep_mut]
    pub input_controller: InputController,
}

pub struct App<T>
where
    T: AppStateHandler,
{
    descriptor: AppDescriptor,
    controllers: Option<Controllers>,
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
            controllers: None,

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

        self.controllers = Some(Controllers {
            graphics_controller: GraphicsController::new(Arc::clone(&window)).unwrap(),
            input_controller: InputController::new(),
        });

        let app_state = T::new(Arc::clone(&window), self.controllers.as_mut().unwrap());
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

        app_state.winit_event(
            WinitEvent::Window(&event),
            self.controllers.as_mut().unwrap(),
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
                self.last_frame = Instant::now();

                // tick handling
                if !self.next_tick.elapsed().is_zero() {
                    app_state.tick(self.last_tick.elapsed(), self.controllers.as_mut().unwrap());

                    self.last_tick = Instant::now();
                    self.next_tick += Duration::from_secs_f32(1.0 / T::TICKS_PER_SECOND);
                    if self.next_tick.elapsed() > Duration::from_secs_f32(20.0 / T::TICKS_PER_SECOND) {
                        self.next_tick = Instant::now() - Duration::from_secs_f32(20.0 / T::TICKS_PER_SECOND)
                    }
                }
                // where the magic happens
                app_state.render(frame_time, self.controllers.as_mut().unwrap());

                // mouse logic
                let new_mouse_locked = self.controllers.as_mut().unwrap().input_controller.is_mouse_locked();
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

                self.controllers.as_mut().unwrap().input_controller.clear_inputs();

                window.request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.controllers.as_mut().unwrap().graphics_controller.resize(new_size);
            }
            WindowEvent::Focused(is_focused) => {
                app_state.window_focus_changed(is_focused, self.controllers.as_mut().unwrap());
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

        app_state.winit_event(
            WinitEvent::Device(&event),
            self.controllers.as_mut().unwrap(),
        )
    }
}
