use clap::Parser;
use log::info;
use rekop_gbc::device::Device;
use rekop_gbc::Result;

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

fn main() -> Result<()> {
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
    Ok(())
}

fn run_device(mut device: Device) {
    
}
