use crate::cli::options;

mod cli;

fn main() {
    if let Err(err) = options().run().run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
