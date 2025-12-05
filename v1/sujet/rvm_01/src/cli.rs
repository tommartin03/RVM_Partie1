use std::ffi::OsString;

use clap::{Parser, Subcommand};


/// We use a macro from the 'clap' crate to generate the cli part of our application 
#[derive(Debug, Parser)]
#[command(name = "rvm")]
#[command(about = "A rust virtual machine executing text assembly files", long_about = None)]
pub struct Cli {
    #[arg(
        short = 'v',
        action = clap::ArgAction:: Count,
        global = true,

    )]
    pub verbosity_counter: u8,
    /// silence logs, override '-v'.
    #[arg(short, long, global = true)]
    pub silence: bool,
        #[command(subcommand)]
        pub command: Command,
}

// Here are the different subcommand (only 1 at the moment :) )
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Take a text assembly file as input and interpret it
    #[command(arg_required_else_help = true)]
    Exec {
        /// The text assembly file to interpret
        path: OsString,
    },
}

impl Cli {
    pub fn get_verbosity_level(&self) -> log::LevelFilter {
        if self.silence {
            log::LevelFilter::Off
        } else {
            match self.verbosity_counter {
                0 => log::LevelFilter::Off,
                1 => log::LevelFilter::Info,
                2 => log::LevelFilter::Debug,
                3 => log::LevelFilter::Trace,
                _ => panic!("Invalid verbosity level"),
            }
        }
    }
}

