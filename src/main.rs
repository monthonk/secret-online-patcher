use clap::Parser;
use secret_online_patcher::cli::{Args, Operation};

fn main() {
    let args = Args::parse();

    match args.op {
        Operation::List => {
            // Call the function to list applications
            secret_online_patcher::cli::list_apps();
        }
        Operation::AddApp => {
            // Call the function to add an app
            secret_online_patcher::cli::add_app();
        }
    }
}
