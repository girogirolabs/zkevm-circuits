#!/usr/bin/env bash

# Set variables for thread counts
LEADER_THREADS=24
WORKER_THREADS=12
SINGLE_NODE_THREADS=48

# Ensure ./benchmark directory exists
mkdir -p ./benchmark

# Start worker processes
echo "Starting worker processes..."
RAYON_NUM_THREADS=$WORKER_THREADS make keccak_worker_0 > benchmark/out_worker_0.txt 2>&1 &
WORKER_PID0=$!
RAYON_NUM_THREADS=$WORKER_THREADS make keccak_worker_1 > benchmark/out_worker_1.txt 2>&1 &
WORKER_PID1=$!
sleep 1

# Start leader process
echo "Starting leader process..."
RAYON_NUM_THREADS=$LEADER_THREADS make keccak_leader > benchmark/out_leader.txt 2>&1 &
LEADER_PID=$!

# Wait for worker and leader processes to complete
echo "Waiting for worker and leader processes to complete..."
wait $WORKER_PID0 $WORKER_PID1 $LEADER_PID

# Start a single-node process
# echo "Starting single-node process..."
# RAYON_NUM_THREADS=$SINGLE_NODE_THREADS make keccak_single > benchmark/out_single.txt 2>&1

# echo "Benchmark completed."
