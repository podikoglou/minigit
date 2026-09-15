use crate::cli::Args;

mod cli;

fn main() {
    let args: Args = argh::from_env();

    if let Err(err) = args.run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
