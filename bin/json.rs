use clap::Parser;
use ebar::TryRun;

fn main() {
    let app = ebar::tools::json::Json::parse();
    let ctx = ebar::Context {};
    if let Err(e) = app.run(&ctx) {
        eprintln!("{}", &e);
    }
}
