#[derive(Debug, thiserror::Error)]
pub enum EmulatorError {
    #[error("CPU error: {0}")]
    Cpu(#[from] CpuError),

    #[error("MMU error: {0}")]
    Mmu(#[from] MmuError),

    #[error("Cartridge error: {0}")]
    Cartridge(#[from] CartridgeError),

    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum CpuError {
    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Stack overflow")]
    StackOverflow,

    #[error("UnknownOpcode {0:#04X}")]
    UnknownOpcode(u8),
}

#[derive(Debug, thiserror::Error)]
pub enum MmuError {
    #[error("Stack underflow")]
    StackUnderflow,
}

#[derive(Debug, thiserror::Error)]
pub enum CartridgeError {
    #[error("Unsupported cartridge type {0:#04x}")]
    UnsupportedType(u8),

    #[error("Invalid ROM size")]
    InvalidRomSize,
}

pub type Result<T> = std::result::Result<T, EmulatorError>;
