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

Features (both enabled by default):
- `asm`: Enables CKB-VM's optimized assembly interpreter
- `use-k256`: Uses the Rust k256 library for ECDSA verification. Disable this to use the C secp256k1 library for better performance.

To build without a specific feature:
```sh
~/.sp1/bin/cargo-prove prove build --no-default-features --features asm
``` 


### Execute the Program

For faster development iteration, use `--minimal-execute` which only outputs cycle count and gas cost:
```sh
cargo run --release -- --minimal-execute --mode vm
cargo run --release -- --minimal-execute --mode native
```

To run the program without generating a proof (slower than above):

```sh
cargo run --release -- --execute --mode vm
cargo run --release -- --execute --mode native
```

### Benchmark

The `--mode` flag accepts `vm` or `native`:
- `vm` (default): Runs k256_ecdsa inside CKB-VM interpreter on SP1
- `native`: Runs k256_ecdsa directly on SP1

| Mode   | SP1 Instructions | SP1 Cycles | CKB-VM Cycles |
|--------|------------------|------------|---------------|
| vm     | 135.07M          | 1080.52M   | 7,770,651     |
| native | 2.62M            | 20.99M     | N/A           |

The VM mode is about 51x slower than native mode.

The benchmarks above use the CKB-VM RV64IM implementation with the Rust k256 library, which is not the optimal approach for ECDSA (secp256k1) signature verification. For better performance, use the C version of secp256k1 by disabling the `use-k256` feature in the program. The results are as follows:

```
SP1 instruction executed: 27.05M
```



### Generate an SP1 Core Proof

To generate an SP1 [core proof](https://docs.succinct.xyz/docs/sp1/generating-proofs/proof-types#core-default):

```sh
cargo run --release -- --prove --mode vm
cargo run --release -- --prove --mode native
```

> **Note:** Proof generation is resource-intensive and may take significant time and memory.

