use clap::Parser;
use ebar::{Context, TryRun};

fn main() {
    let app = ebar::App::parse();
    let ctx = Context {};
    if let Err(e) = app.run(&ctx) {
        eprintln!("{}", &e);
    }
}
