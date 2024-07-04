use std::env;

mod circuits;
mod util;

enum Command {
    Setup,
    Prove,
    ProveLocal,
    Verify,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = match args.get(1).map(|s| s.as_str()) {
        Some("setup") => Command::Setup,
        Some("prove") => Command::Prove,
        Some("prove-local") => Command::ProveLocal,
        Some("verify") => Command::Verify,
        _ => {
            eprintln!("Usage: {} <setup|prove|verify>", args[0]);
            std::process::exit(1);
        }
    };

    match command {
        Command::Setup => {
            circuits::evm::setup();
        }
        Command::Prove => {
            let prover_index = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(0);
            circuits::evm::prove(prover_index);
        }
        Command::ProveLocal => {
            circuits::evm::prove_local();
        }
        Command::Verify => {
            circuits::evm::verify();
        }
    }
}
