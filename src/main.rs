use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use wal::WriteAheadLog;
mod byte_encoder;
pub mod clock;
pub mod column_store;
pub mod column_value;
pub mod storage;
pub mod wal;

pub struct TimeSeriesDatabase {
    data: BTreeMap<SystemTime, f64>,
    wal: WriteAheadLog,
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

fn main() {
    let cli = Cli::parse();
}
