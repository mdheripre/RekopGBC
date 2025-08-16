use clap::Parser;
use rekop_gbc::device::Device;

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

fn main() {
    let args = Args::parse();

    let device = Device::new(&args.rom, args.save_state);
}
