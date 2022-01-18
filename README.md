<p align="center">
  <img src="https://user-images.githubusercontent.com/26602057/144711927-22cb7b89-6eb3-4794-b8e1-ac455f0957b1.png" width="200">
</p>

<div align="center">
<h1>Web3Games</h1>
</div>

<!-- TOC -->

- [1. Introduction](#1-introduction)
- [2. Building](#2-building)
- [3. Run](#3-run)
- [4. Docker](#4-run-in-docker)

<!-- /TOC -->

## 1. Introduction

Web3Games is a new generation gaming ecosystem built on Substrate.

## 2. Building

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

## 3. Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/web3games-node purge-chain --dev
```

Start a development chain with:

```bash
./target/release/web3games-node --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator
```

## 4. Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command
(`cargo build --release && ./target/release/web3games-node --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/web3games-node --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/web3games-node purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```

## License

Web3Games is released under the [GPL v3.0 License](LICENSE).
