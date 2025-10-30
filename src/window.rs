use std::num::NonZeroU32;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize};
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
    window: Option<Arc<Window>>,
    sender: Sender<GBEvent>,
    pub receiver: Receiver<Vec<u32>>,
    data: Option<Vec<u32>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
}

impl App {
    pub fn new(sender: Sender<GBEvent>, receiver: Receiver<Vec<u32>>) -> App {
        App {
            window: None,
            sender,
            receiver,
            data: None,
            surface: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("RekopGBC")
                        .with_inner_size(PhysicalSize::new(160, 144))
                        .with_resizable(false),
                )
                .unwrap(),
        );

        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();
        self.surface = Some(surface);
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;

        while let Ok(data) = self.receiver.try_recv() {
            self.data = Some(data);
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
                if let Some(data) = &self.data {
                    if let Some(window) = &self.window {
                        if let Some(surface) = &mut self.surface {
                            let (width, height) = {
                                let size = window.inner_size();
                                (size.width, size.height)
                            };

                            surface
                                .resize(
                                    NonZeroU32::new(width).unwrap(),
                                    NonZeroU32::new(height).unwrap(),
                                )
                                .unwrap();

                            surface.buffer_mut().unwrap().copy_from_slice(data);
                        }
                    }
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
