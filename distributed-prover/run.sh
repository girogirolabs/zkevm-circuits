#!/bin/bash

# Function to display usage
usage() {
  echo "Usage: $0 [--profile dev|release] <circuit> <setup|prove|prove-local|verify> [prover_index]"
  exit 1
}

# Initialize variables
PROFILE="release"
CIRCUIT=""
COMMAND=""
PROVER_INDEX=""

# Parse command line arguments
if [[ "$#" -lt 1 ]]; then
  usage
fi

while [[ "$#" -gt 0 ]]; do
  case $1 in
    --profile)
      PROFILE="$2"
      shift
      ;;
    evm|keccak)
      if [[ -n "$CIRCUIT" ]]; then
        usage
      else
        CIRCUIT="$1"
      fi
      ;;
    setup|prove|prove-local|verify)
      if [[ -n "$COMMAND" ]]; then
        PROVER_INDEX="$1"
      else
        COMMAND="$1"
      fi
      ;;
    *)
      PROVER_INDEX="$1"
      ;;
  esac
  shift
done

# Check if command is provided
if [[ -z "$COMMAND" ]]; then
  usage
fi

# Check if circuit is provided
if [[ -z "$CIRCUIT" ]]; then
  usage
fi

# Validate profile
if [[ "$PROFILE" != "dev" && "$PROFILE" != "release" ]]; then
  echo "Invalid profile: $PROFILE. Must be 'dev' or 'release'."
  exit 1
fi

# Validate command
if [[ "$COMMAND" != "setup" && "$COMMAND" != "prove" && "$COMMAND" != "prove-local" && "$COMMAND" != "verify" ]]; then
  echo "Invalid command: $COMMAND. Must be 'setup', 'prove', 'prove-local', or 'verify'."
  exit 1
fi

# Validate circuit
if [[ "$CIRCUIT" != "evm" && "$CIRCUIT" != "keccak" ]]; then
  echo "Invalid circuit: $CIRCUIT. Must be 'evm' or 'keccak'."
  exit 1
fi

# If command is prove, check for prover index
if [[ "$COMMAND" == "prove" && -z "$PROVER_INDEX" ]]; then
  echo "Prover index is required for the 'prove' command."
  exit 1
fi

# Construct the cargo command
if [[ "$COMMAND" == "prove" ]]; then
  cargo run --package distributed-prover --profile "$PROFILE" "$CIRCUIT" "$COMMAND" "$PROVER_INDEX"
else
  cargo run --package distributed-prover --profile "$PROFILE" "$CIRCUIT" "$COMMAND"
fi
