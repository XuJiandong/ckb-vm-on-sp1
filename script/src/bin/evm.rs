//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system plonk
//! ```

use alloy_sol_types::SolType;
use ckb_vm_interpreter_lib::PublicValuesStruct;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    include_elf, Elf, HashableKey, ProveRequest, Prover, ProverClient,
    ProvingKey as ProvingKeyTrait, SP1ProofWithPublicValues, SP1Stdin,
};
use std::path::PathBuf;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const CKB_VM_INTERPRETER_ELF: Elf = include_elf!("ckb-vm-interpreter-program");

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct EVMArgs {
    #[arg(long, default_value = "20")]
    n: u32,
    #[arg(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1ProofFixture {
    a: u32,
    b: u32,
    n: u32,
    vkey: String,
    public_values: String,
    proof: String,
}

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();

    let args = EVMArgs::parse();
    let client = ProverClient::from_env().await;
    let pk = client
        .setup(CKB_VM_INTERPRETER_ELF)
        .await
        .expect("setup failed");

    let mut stdin = SP1Stdin::new();
    stdin.write(&args.n);

    println!("n: {}", args.n);
    println!("Proof System: {:?}", args.system);

    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, stdin).plonk().await,
        ProofSystem::Groth16 => client.prove(&pk, stdin).groth16().await,
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, &pk, args.system);
}

/// Create a fixture for the given proof.
fn create_proof_fixture<PK: ProvingKeyTrait>(
    proof: &SP1ProofWithPublicValues,
    pk: &PK,
    system: ProofSystem,
) {
    let bytes = proof.public_values.as_slice();
    let PublicValuesStruct { n, a, b } = PublicValuesStruct::abi_decode(bytes).unwrap();

    let fixture = SP1ProofFixture {
        a,
        b,
        n,
        vkey: pk.verifying_key().bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    println!("Verification Key: {}", fixture.vkey);
    println!("Public Values: {}", fixture.public_values);
    println!("Proof Bytes: {}", fixture.proof);

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
