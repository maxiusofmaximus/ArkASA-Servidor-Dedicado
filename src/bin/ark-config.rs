// ARK ASA Config Manager CLI
// Usage: ark-config <COMMAND> [OPTIONS]

use ark_asa_config::cli::CLI;
use std::env;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    match CLI::parse_args(args) {
        Ok(command) => match CLI::execute(command).await {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            CLI::print_help();
            std::process::exit(1);
        }
    }
}
