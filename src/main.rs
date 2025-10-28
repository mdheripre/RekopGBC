use anyhow::{Error, anyhow};
use clap::Parser;
use log::info;
use rekop_gbc::{device::Device, window::App};
use winit::{event_loop::{self, EventLoop}};

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
    let mut device = Device::new(&args.rom, args.save_state)?;
    std::thread::spawn(move || {
        device.do_cycle()
    });

    run_window()
        .map_err(|e| {
            eprintln!("{e}");
            e
        })?;

    Ok(())
}

fn run_window() -> Result<(), Error> {
    let event_loop = EventLoop::new().expect("Failed to create event Loop");
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).map_err(|e| anyhow!(e))
}
