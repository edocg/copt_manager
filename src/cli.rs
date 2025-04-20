use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "COPT CLI")]
#[command(about = "Sistema de pagos COPT para residentes", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    AddResident {
        #[arg(long)]
        id: u32,
        #[arg(long)]
        name: String,
        #[arg(long)]
        wallet: String,
    },
    Charge {
        #[arg(long)]
        id: u32,
        #[arg(long)]
        amount: u64,
    },
    Report {
        #[arg(long)]
        id: Option<u32>,
    },
}