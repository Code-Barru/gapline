use clap::Parser;
use headway::Cli;

fn main() {
    let args = Cli::parse();

    println!("{args:?}");
    println!("Not implemented yet!");
}
