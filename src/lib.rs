pub mod tools;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::Parser;

pub struct Context {}

impl Context {
    pub fn full_extend(&self, path: &str) -> Result<PathBuf> {
        let full = shellexpand::full(path)?;
        Ok(Path::new(full.as_ref()).to_path_buf())
    }

    pub fn read_to_string(&self, path: &str) -> Result<String> {
        let full = shellexpand::full(path)?;
        let path = Path::new(full.as_ref());
        let text = fs::read_to_string(path)?;
        Ok(text)
    }
}

pub trait TryRun {
    type Err;

    fn run(&self, ctx: &Context) -> Result<(), Self::Err>;
}

#[macro_export]
macro_rules! define_sub_commands {
    ($name:ident, $($command:ident),+) => {
        #[derive(Debug, clap::Subcommand)]
        pub enum $name {
			$(
				$command($command),
			)+
        }

        impl crate::TryRun for $name {
			type Err = anyhow::Error;
            fn run(&self, ctx: &crate::Context) -> Result<(), Self::Err> {
                match self {
					$(
                    	$name::$command(c) => c.run(ctx),
					)+
                }
            }
        }
    };
}

/// Validates, formats or searchs the json file.
#[derive(Debug, Parser)]
pub struct App {
    #[clap(subcommand)]
    sub_commands: SubCommands,
}

use tools::json::Json;

define_sub_commands! {SubCommands, Json}

impl crate::TryRun for App {
    type Err = anyhow::Error;

    fn run(&self, ctx: &crate::Context) -> Result<(), Self::Err> {
        self.sub_commands.run(ctx)
    }
}
