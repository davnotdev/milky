use super::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::time::Duration;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub use winit::event::WindowEvent;

pub struct WindowPlanet {
    event_loop: Option<EventLoop<()>>,
    pub(super) window_event_radio: Radio<WindowEvent>,
    window: Window,
}

//  `event_loop` is not Send / Sync, but we move it out before we need Send / Sync.
unsafe impl Send for WindowPlanet {}
unsafe impl Sync for WindowPlanet {}

impl WindowPlanet {
    pub fn new() -> WindowPlanet {
        let event_loop = EventLoop::new().unwrap();
        let window = Window::new(&event_loop).unwrap();
        WindowPlanet {
            window_event_radio: Radio::new(),
            event_loop: Some(event_loop),
            window,
        }
    }

    pub fn take_event_loop(&mut self) -> EventLoop<()> {
        self.event_loop
            .take()
            .unwrap_or_else(|| panic!("WindowPlanet's take_event_loop can only be called once"))
    }

    pub fn get_radio(&self) -> &Radio<WindowEvent> {
        &self.window_event_radio
    }

    pub fn flush_radios(&mut self) {
        self.window_event_radio.flush();
    }

    pub fn set_title(&self, name: &str) {
        self.window.set_title(name);
    }

    pub fn get_size(&self) -> (usize, usize) {
        let size = self.window.inner_size();
        (size.width as usize, size.height as usize)
    }

    pub fn get_raw_display_handle(&self) -> RawDisplayHandle {
        self.window.display_handle().unwrap().as_raw()
    }

    pub fn get_raw_window_handle(&self) -> RawWindowHandle {
        self.window.window_handle().unwrap().as_raw()
    }
}

pub fn run_window_loop<T: Send + Sync>(rt: &mut Runtime<T>) {
    let event_loop = rt.planets.window.take_event_loop();

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::wait_duration(Duration::from_millis(10)));
            match event {
                Event::WindowEvent {
                    event,
                    window_id: _,
                } => match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::RedrawRequested => {}
                    _ => {
                        rt.planets.window.get_radio().send(event);
                    }
                },
                Event::AboutToWait => {
                    rt.tick();
                    rt.planets.window.flush_radios();
                }
                _ => (),
            }
        })
        .unwrap();
}
