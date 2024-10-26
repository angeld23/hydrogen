use anyhow::Result;
use hydrogen_core::app::{App, AppDescriptor, AppStateHandler, Controllers};
use std::{ops::Range, sync::Arc, time::Duration};
use winit::{event_loop::EventLoop, window::Window};

struct AppState {}

impl AppStateHandler for AppState {
    fn new(window: Arc<Window>, controllers: &mut Controllers) -> Self {
        Self {}
    }
    fn render(&mut self, delta: Duration, controllers: &mut Controllers) {
        controllers.input_controller.report_in_a_menu();
    }
    fn window_focus_changed(&mut self, focused: bool, controllers: &mut Controllers) {
        println!("{}", focused);
    }
}

fn main() -> Result<()> {
    env_logger::builder().format_timestamp(None).init();

    let mut app = App::<AppState>::new(AppDescriptor {
        window_title: "h1_test".into(),
    });

    EventLoop::new().unwrap().run_app(&mut app)?;

    Ok(())
}
