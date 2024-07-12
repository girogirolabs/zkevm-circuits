use std::env;

use env_logger::{Builder, Target};

use circuit_helper::{
    Circuit,
    circuits::{
        common::CircuitHelper,
        evm::EvmCircuitHelper,
        keccak::KeccakCircuitHelper,
    }
};

enum Command {
    Setup,
    Prove,
    ProveLocal,
    Verify,
}

fn main() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage: {} <evm|keccak> <setup|prove|prove-local|verify> [prover_index]", args[0]);

    let circuit = match args.get(1).map(|s| s.as_str()) {
        Some("evm") => Circuit::EVM,
        Some("keccak") => Circuit::Keccak,
        _ => {
            eprintln!("{}", usage);
            std::process::exit(1);
        }
    };
    let command = match args.get(2).map(|s| s.as_str()) {
        Some("setup") => Command::Setup,
        Some("prove") => Command::Prove,
        Some("prove-local") => Command::ProveLocal,
        Some("verify") => Command::Verify,
        _ => {
            eprintln!("{}", usage);
            std::process::exit(1);
        }
    };

    match circuit {
        Circuit::EVM => match command {
            Command::Setup => {
                EvmCircuitHelper::setup();
            }
            Command::Prove => {
                let prover_index = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(0);
                EvmCircuitHelper::prove(prover_index);
            }
            Command::ProveLocal => {
                EvmCircuitHelper::prove_local();
            }
            Command::Verify => {
                EvmCircuitHelper::verify();
            }
        },
        Circuit::Keccak => match command {
            Command::Setup => {
                KeccakCircuitHelper::setup();
            }
            Command::Prove => {
                let prover_index = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(0);
                KeccakCircuitHelper::prove(prover_index);
            }
            Command::ProveLocal => {
                KeccakCircuitHelper::prove_local();
            }
            Command::Verify => {
                KeccakCircuitHelper::verify();
            }
        }
    }
}
