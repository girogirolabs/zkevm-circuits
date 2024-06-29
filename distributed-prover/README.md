# Distributed prover for zkevm-circuits

First, run setup.

```bash
./run.sh setup
```

Then generate proof with one local prover (baseline).

```bash
./run.sh prove-local
```

Alternatively, generate proof with distributed provers.

By default, there will be two provers running on localhost on the same node. This can be changed by modifying `network_config.json`.

```bash
# On node 0
./run.sh prove 0
# On node 1
./run.sh prove 1
```

Finally, verify the proof.

```bash
./run.sh verify
```


