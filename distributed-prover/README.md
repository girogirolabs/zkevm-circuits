# Distributed prover demo for zkevm-circuits

We will use the keccak circuit as an example.

First, run setup.

```bash
RUST_LOG=info cargo run --release -- keccak setup
```

Run the local prover first for baseline results.

```bash
RUST_LOG=info cargo run --release -- keccak prove-local
```

To enable GPU acceleration, set the environment variable `ICICLE_GPU` and enable the `gpu` feature flag.

```bash
RUST_LOG=info ICICLE_GPU=1 cargo run --release --features gpu -- keccak prove-local
```

Next, run distributed provers. The network and workload configurations for distributed proving are specified in two files.

For example, for keccak, they are [network_config.json](../circuit-helper/artifacts/keccak/network_config.json) and [workload_config.json](../circuit-helper/artifacts/keccak/workload_config.json).

By default, there are two provers, both running on localhost. To run them (with GPU acceleration), open up two terminals.

```bash
# In termial 0
RUST_LOG=info ICICLE_GPU=1 cargo run --release --features gpu -- keccak prove 0

# In terminal 1
RUST_LOG=info ICICLE_GPU=1 cargo run --release --features gpu -- keccak prove 1
```

For meaningful performance metrics, those provers should be run on different machines. To do so, modify [network_config.json](../circuit-helper/artifacts/keccak/network_config.json) with new prover URLs and run the commands above on each machine.

Finally, verify the proof. This should be run on the leader prover's machine (prover 0).

```bash
RUST_LOG=info cargo run --release -- keccak verify
```
