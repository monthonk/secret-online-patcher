use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
pub struct Args {
    /// Operation to perform
    #[arg(help = "Operation to perform")]
    pub op: Operation,
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "kebab_case")]
pub enum Operation {
    List,
    AddApp,
}

pub fn list_apps() {
    // Implementation for listing apps
    println!("Listing applications...");
}

pub fn add_app() {
    // Implementation for adding an app
    println!("Adding a new application...");
}
