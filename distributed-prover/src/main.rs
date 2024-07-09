use std::env;

mod circuits;
mod util;

enum Circuit {
    EVM,
    Keccak,
}

enum Command {
    Setup,
    Prove,
    ProveLocal,
    Verify,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let circuit = match args.get(1).map(|s| s.as_str()) {
        Some("evm") => Circuit::EVM,
        Some("keccak") => Circuit::Keccak,
        _ => {
            eprintln!("Usage: {} <evm|keccak> <setup|prove|prove-local|verify>", args[0]);
            std::process::exit(1);
        }
    };
    let command = match args.get(2).map(|s| s.as_str()) {
        Some("setup") => Command::Setup,
        Some("prove") => Command::Prove,
        Some("prove-local") => Command::ProveLocal,
        Some("verify") => Command::Verify,
        _ => {
            eprintln!("Usage: {} <evm|keccak> <setup|prove|prove-local|verify>", args[0]);
            std::process::exit(1);
        }
    };

    match circuit {
        Circuit::EVM => match command {
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
        Circuit::Keccak => match command {
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
    }
}
