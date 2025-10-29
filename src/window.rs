use std::sync::mpsc::{Receiver, Sender};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

pub enum GBEvent {
    ArrowDown,
    ArrowUp,
}

pub struct App {
    window: Option<Window>,
    sender: Sender<GBEvent>,
    pub receiver: Receiver<Vec<u8>>,
    data: Option<Vec<u8>>,
}

impl App {
    pub fn new(sender: Sender<GBEvent>, receiver: Receiver<Vec<u8>>) -> App {
        App {
            window: None,
            sender,
            receiver,
            data: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes().with_title("RekopGBC"))
                .unwrap(),
        )
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;

        while let Ok(data) = self.receiver.try_recv() {
            self.data = Some(data);
            // TODO: transform Vec<u8> into pixels
        }

        if self.data.is_some() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &self.data {
                    // TBD pixel rendering
                }
            }
            WindowEvent::Resized(size) => {
                println!("Resized: {}, {}", size.width, size.height)
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                _ = is_synthetic;
                _ = device_id;

                match event.physical_key {
                    PhysicalKey::Code(KeyCode::ArrowUp) => {
                        if self.sender.send(GBEvent::ArrowUp).is_err() {
                            eprintln!("Send error: backend disconnected, exiting..");
                            event_loop.exit();
                        }
                    }
                    PhysicalKey::Code(KeyCode::ArrowDown) => {
                        if self.sender.send(GBEvent::ArrowDown).is_err() {
                            eprintln!("Send error: backend disconnected, exiting..");
                            event_loop.exit();
                        }
                    }
                    _ => {}
                }
                println!("Keyboard key pressed: {:?}", event.physical_key)
            }
            _ => (),
        }
    }
}
