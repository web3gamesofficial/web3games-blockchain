#!/usr/bin/env bash

cargo build --release --features="runtime-benchmarks"

target/release/web3games-node benchmark pallet --chain=$1 --list | sed -n '2,$p' | grep -Eio "^\w+" | uniq |
  while IFS= read -r line
  do
      pallet=$line
      temp=${pallet/pallet_/}
      pallet_dir=${temp//_/-}
      if [ "$pallet" != "frame_system" -a "$pallet" != "pallet_balances" -a "$pallet" != "pallet_timestamp" ]; then
          echo "benchmark ${pallet}"
          target/release/web3games-node benchmark pallet --chain=$1 \
          --steps=50 \
          --repeat=20 \
          --pallet=$pallet \
          --extrinsic="*" \
          --execution=wasm \
          --wasm-execution=compiled \
          --output="./pallets/${pallet_dir}/src/weights.rs" \
          --template="./.maintain/w3g-weight-template.hbs";
      fi
  done
