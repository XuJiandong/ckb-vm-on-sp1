# CKB-VM Interpreter on SP1

This project runs a [CKB-VM](https://github.com/nervosnetwork/ckb-vm) interpreter inside the [SP1](https://github.com/succinctlabs/sp1) zkVM to generate zero-knowledge proofs of CKB-VM program execution.

The example program performs a secp256k1 ECDSA signature verification inside CKB-VM, which itself runs within SP1 zkVM.

## Dependencies

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/docs/sp1/getting-started/install) (v6.0.0-beta.1)

## Project Structure

- `program/` - The SP1 zkVM program that runs the CKB-VM interpreter
- `script/` - Scripts to build, execute, and generate proofs
- `lib/` - Shared library for public values

## Running the Project

### Build the Program

```sh
cd program
~/.sp1/bin/cargo-prove prove build
```

Or it will be automatically built through `script/build.rs` when the script is built.

### Execute the Program

To run the program without generating a proof:

```sh
cargo run --release -- --execute
```

For faster development iteration, use `--minimal-execute` which only outputs cycle count:

```sh
cargo run --release -- --minimal-execute
```

### Generate an SP1 Core Proof

To generate an SP1 [core proof](https://docs.succinct.xyz/docs/sp1/generating-proofs/proof-types#core-default):

```sh
cargo run --release -- --prove
```


### Retrieve the Verification Key

```sh
cargo run --release --bin vkey
```

## Using the Prover Network

We recommend using the [Succinct Prover Network](https://docs.succinct.xyz/docs/network/introduction) for non-trivial programs. See the [key setup guide](https://docs.succinct.xyz/docs/network/developers/key-setup) to get started.

```sh
cp .env.example .env
```

Set `SP1_PROVER=network` and `NETWORK_PRIVATE_KEY` to your whitelisted private key.

```sh
SP1_PROVER=network NETWORK_PRIVATE_KEY=... cargo run --release --bin evm
```
