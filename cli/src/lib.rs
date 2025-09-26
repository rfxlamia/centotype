//! # Centotype CLI
//!
//! Command-line interface with interactive navigation and menu systems.

pub mod commands;
pub mod interface;
pub mod navigation;
pub mod menus;

use centotype_core::types::*;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "centotype")]
#[command(about = "CLI-based typing trainer with 100 progressive difficulty levels")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start arcade mode training
    Play {
        /// Level to play (1-100)
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=100))]
        level: Option<u8>,
    },
    /// Practice specific skills
    Drill {
        /// Category to practice
        #[arg(short, long)]
        category: String,
        /// Duration in minutes
        #[arg(short, long, default_value_t = 5)]
        duration: u32,
    },
    /// Endurance training session
    Endurance {
        /// Duration in minutes
        #[arg(short, long, default_value_t = 15)]
        duration: u32,
    },
    /// View statistics and progress
    Stats,
    /// Configure application settings
    Config,
}

pub struct CliManager {
    // CLI implementation
}

impl CliManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Play { level } => {
                println!("Starting arcade mode, level: {:?}", level);
            }
            Commands::Drill { category, duration } => {
                println!("Starting drill: {} for {} minutes", category, duration);
            }
            Commands::Endurance { duration } => {
                println!("Starting endurance mode for {} minutes", duration);
            }
            Commands::Stats => {
                println!("Displaying statistics");
            }
            Commands::Config => {
                println!("Opening configuration");
            }
        }
        Ok(())
    }
}