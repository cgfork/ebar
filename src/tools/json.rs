use clap::{Args, Parser};
use serde_json::Value;
use view::ViewPath;

/// Searchs with the specified path from the json file if the path
/// is specified. Otherwise, show the pretty json string.
#[derive(Debug, Args)]
pub struct Search {
    /// if --path or -p is specified, searchs from the json file.
    #[clap(long, short)]
    path: Option<String>,

    #[clap(name = "json-file")]
    file: String,
}

impl crate::TryRun for Search {
    type Err = anyhow::Error;

    fn run(&self, ctx: &crate::Context) -> Result<(), Self::Err> {
        let data = ctx.read_to_string(&self.file)?;
        let value = serde_json::from_str::<Value>(&data)?;
        match if let Some(path) = &self.path {
            ejson::search_path(&value, ViewPath::parse_str(path)?)
        } else {
            Some(&value)
        } {
            Some(v) => {
                println!("{}", &serde_json::to_string_pretty(v)?);
                Ok(())
            }
            None => Ok(()),
        }
    }
}

/// Resolves the path for the target value from the json file.
#[derive(Debug, Args)]
pub struct Resolve {
    /// The target value to resolve.
    #[clap(long, short)]
    target: Option<String>,

    /// The regex target value to resolve.
    #[clap(long, short, conflicts_with = "target")]
    regex: Option<String>,

    #[clap(name = "json-file")]
    file: String,
}

impl crate::TryRun for Resolve {
    type Err = anyhow::Error;

    fn run(&self, ctx: &crate::Context) -> Result<(), Self::Err> {
        let data = ctx.read_to_string(&self.file)?;
        let value = serde_json::from_str::<Value>(&data)?;
        if let Some(target) = &self.target {
            match ejson::nest_find_value(&value, target) {
                Some(paths) => {
                    for path in paths {
                        println!("{}", &path)
                    }
                }
                None => println!("{} is not found", target),
            }
        } else if let Some(target) = &self.regex {
            let reg = regex::Regex::new(target)?;
            match ejson::nest_find_regex(&value, reg) {
                Some(paths) => {
                    for (path, v) in paths {
                        println!("{}: {}", &path, v)
                    }
                }
                None => println!("{} is not found", target),
            }
        }
        Ok(())
    }
}

/// Validates, formats or searchs the json file.
#[derive(Debug, Parser)]
pub struct Json {
    #[clap(subcommand)]
    sub_commands: SubCommands,
}

crate::define_sub_commands! {SubCommands, Search, Resolve}

impl crate::TryRun for Json {
    type Err = anyhow::Error;

    fn run(&self, ctx: &crate::Context) -> Result<(), Self::Err> {
        self.sub_commands.run(ctx)
    }
}
