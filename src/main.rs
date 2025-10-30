use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError, TrySendError};

use anyhow::{anyhow, Error};
use clap::Parser;
use log::info;
use rekop_gbc::{
    device::Device,
    window::{App, GBEvent},
};
use winit::event_loop::{self, EventLoop};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "rekop-gbc")]
struct Args {
    rom: String,

    #[arg(short, long, value_name = "FILE")]
    save_state: Option<String>,

    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    if args.debug {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    info!("Starting emulator ...");
    info!("Creating device ...");
    let device = Device::new(&args.rom, args.save_state)?;
    let (sender1, receiver1) = mpsc::channel();
    let (sender2, receiver2) = mpsc::sync_channel(1);
    let device_thread = std::thread::spawn(move || run_device(device, sender2, receiver1));

    run_window(sender1, receiver2).map_err(|e| {
        eprintln!("{e}");
        e
    })?;

    let _ = device_thread.join();
    Ok(())
}

fn run_device(mut device: Device, sender: SyncSender<Vec<u8>>, receiver: Receiver<GBEvent>) {
    'outer: loop {
        device.do_cycle();
        let data = device.ppu_data();
        if let Err(TrySendError::Disconnected(..)) = sender.try_send(data) {
            eprintln!("Send error: frontend disconnected, exiting..");
            break 'outer;
        }

        'recv: loop {
            match receiver.try_recv() {
                Ok(event) => match event {
                    GBEvent::ArrowUp => {}
                    GBEvent::ArrowDown => {}
                },
                Err(TryRecvError::Empty) => break 'recv,
                Err(TryRecvError::Disconnected) => {
                    eprintln!("Recv error: frontend disconnected, exiting..");
                    break 'outer;
                }
            }
        }
    }
}

fn run_window(sender: Sender<GBEvent>, receiver: Receiver<Vec<u8>>) -> Result<(), Error> {
    let event_loop = EventLoop::new().expect("Failed to create event Loop");
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    let mut app = App::new(sender, receiver);
    let res = event_loop.run_app(&mut app).map_err(|e| anyhow!(e));
    drop(app.receiver);

    res
}
