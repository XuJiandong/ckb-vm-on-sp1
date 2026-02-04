# CKB-VM on SP1

This project runs [CKB-VM](https://github.com/nervosnetwork/ckb-vm) inside the [SP1](https://github.com/succinctlabs/sp1) zkVM to generate zero-knowledge proofs of CKB-VM program execution. The goal is to estimate the performance of CKB-VM on SP1.

The example program performs secp256k1 ECDSA signature verification inside CKB-VM, which runs within SP1 zkVM.

## Dependencies

- [Rust](https://rustup.rs/) (1.92.0)
- [SP1](https://docs.succinct.xyz/docs/next/sp1/introduction) (v6.0.0-beta.1, hypercube)
- [Protobuf Compiler](https://github.com/protocolbuffers/protobuf) (v29.4)
- gcc-riscv64-linux-gnu (apt)
- rustup target: `riscv64gc-unknown-linux-gnu`

## Project Structure

- `program/` - The SP1 zkVM program that runs the CKB-VM
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

For faster development iteration, use `--minimal-execute` which only outputs cycle count and gas cost:
```sh
cargo run --release -- --minimal-execute
```

To run the program without generating a proof(slower than above):

```sh
cargo run --release -- --execute
```

### Generate an SP1 Core Proof

To generate an SP1 [core proof](https://docs.succinct.xyz/docs/sp1/generating-proofs/proof-types#core-default):

```sh
cargo run --release -- --prove
```

> **Note:** Proof generation is resource-intensive and may take significant time and memory.

