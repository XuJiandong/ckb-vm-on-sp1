# SP1 Program

This is an SP1 zkVM program that runs a CKB-VM interpreter to execute a secp256k1 ECDSA signature verification.

## Dependencies

- SP1 zkVM: `v6.0.0-beta.1` (git)
- CKB-VM: Custom fork for RISC-V compatibility

## Description

The program `src/secp256k1_ecdsa_ckbvm` is a compiled RISC-V binary from the [CKB-VM test suite](https://github.com/nervosnetwork/ckb-vm-contrib/tree/main/ckb-vm-test-suite/programs/contracts/secp256k1_ecdsa). It performs a single secp256k1 ECDSA signature verification inside the CKB-VM, which itself runs within the SP1 zkVM. Uses the C version of secp256k1.

The program `src/k256_ecdsa_ckbvm` is a compiled RISC-V binary from the [CKB-VM test suite](https://github.com/nervosnetwork/ckb-vm-contrib/tree/main/ckb-vm-test-suite/programs/contracts/secp256k1_ecdsa). It performs a single secp256k1 ECDSA signature verification inside the CKB-VM, which itself runs within the SP1 zkVM. Uses the Rust k256 crate.

The program `script/binaries/k256_ecdsa_sp1` is a compiled RISC-V binary from the [CKB-VM test suite](https://github.com/nervosnetwork/ckb-vm-contrib/tree/main/ckb-vm-test-suite/programs/contracts/secp256k1_ecdsa_sp1). It performs a single secp256k1 ECDSA signature verification within the SP1 zkVM. Uses the Rust k256 crate.


## Build

```bash
~/.sp1/bin/cargo-prove prove build
```

## Run

Execute (without proof):
```bash
cargo run --release -- --execute
```

Generate proof:
```bash
cargo run --release -- --prove
```
