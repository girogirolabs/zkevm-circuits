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
    let circuit_name = args.get(1).map(|s| s.as_str()).unwrap();
    let command = match args.get(2).map(|s| s.as_str()) {
        Some("setup") => Command::Setup,
        Some("prove") => Command::Prove,
        Some("prove-local") => Command::ProveLocal,
        Some("verify") => Command::Verify,
        _ => {
            eprintln!("Usage: {} <evm|keccak> <setup|prove|verify>", args[0]);
            std::process::exit(1);
        }
    };

    println!("circuit name: {}; command: {};", circuit_name, match command {
        Command::Setup => "setup",
        Command::Prove => "prove",
        Command::ProveLocal => "prove-local",
        Command::Verify => "verify",
    });
    
    match circuit_name {
        "evm" => match command {
            Command::Setup => {
                circuits::evm::setup();
            }
            Command::Prove => {
                let prover_index = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(0);
                circuits::evm::prove(prover_index);
            }
            Command::ProveLocal => {
                circuits::evm::prove_local();
            }
            Command::Verify => {
                circuits::evm::verify();
            }
        },
        "keccak" => match command {
            Command::Setup => {
                circuits::keccak::setup();
            }
            Command::Prove => {
                let prover_index = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(0);
                circuits::keccak::prove(prover_index);
            }
            Command::ProveLocal => {
                circuits::keccak::prove_local();
            }
            Command::Verify => {
                circuits::keccak::verify();
            }
        }
        _ => {
            eprintln!("Unknown module: {}", circuit_name);
            eprintln!("Usage: {} <evm|keccak> <setup|prove|prove-local|verify>", args[0]);
            std::process::exit(1); 
        }
    }
}
