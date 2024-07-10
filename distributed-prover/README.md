# Distributed prover demo for zkevm-circuits

We will use the keccak circuit as an example.

First, run setup.

```bash
./run.sh keccak setup
```

Run the local prover first for baseline results.

```bash
./run.sh keccak prove-local
```

Next, run distributed provers. The network and workload configurations for distributed proving are specified in two files.

For example, for keccak, they are [network_config.json](../circuit-helper/artifacts/keccak/network_config.json) and [workload_config.json](../circuit-helper/artifacts/keccak/workload_config.json).

By default, there are two provers, both running on localhost. To run them, open up two terminals.

```bash
# In termial 0
./run.sh keccak prove 0

# In terminal 1
./run.sh keccak prove 1
```

For meaningful performance metrics, those provers should be run on different machines. To do so, modify [network_config.json](../circuit-helper/artifacts/keccak/network_config.json) with new prover URLs and run the commands above on each machine.

Finally, verify the proof. This should be run on the leader prover's machine (prover 0).

```bash
./run.sh keccak verify
```
