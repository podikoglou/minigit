use std::error::Error;

use crate::cli::Args;

mod cli;

fn main() {
    let args: Args = argh::from_env();

    if let Err(err) = args.run() {
        eprintln!("error: {err}");
        let mut source = err.source();
        while let Some(cause) = source {
            eprintln!("  caused by: {cause}");
            source = cause.source();
        }
        std::process::exit(1);
    }
}
