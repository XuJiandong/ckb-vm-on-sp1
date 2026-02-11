//! SP1 script for executing and proving the CKB-VM interpreter program.
//!
//! Usage:
//! ```shell
//! # Fast execution with gas estimation (recommended for development)
//! cargo run --release -- --minimal-execute
//!
//! # Full execution via ProverClient
//! RUST_LOG=info cargo run --release -- --execute
//!
//! # Generate proof (requires significant resources)
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use sha2::Digest;
use sp1_core_executor::{GasEstimatingVM, MinimalExecutor, Program, SP1CoreOpts};
use sp1_hypercube::air::PROOF_NONCE_NUM_WORDS;
use sp1_sdk::{
    include_elf, Elf, ProveRequest, Prover, ProverClient, ProvingKey, SP1PublicValues, SP1Stdin,
};
use std::sync::Arc;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const CKB_VM_INTERPRETER_ELF: Elf = include_elf!("ckb-vm-interpreter-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long)]
    minimal_execute: bool,
}

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let args = Args::parse();

    let options_count = args.execute as u8 + args.prove as u8 + args.minimal_execute as u8;
    if options_count != 1 {
        eprintln!(
            "Error: You must specify exactly one of --execute, --prove, or --minimal-execute"
        );
        std::process::exit(1);
    }

    if args.minimal_execute {
        let elf_bytes: &[u8] = &CKB_VM_INTERPRETER_ELF;
        let program = Arc::new(Program::from(elf_bytes).unwrap());
        let mut executor = MinimalExecutor::new(program.clone(), false, Some(1000));

        executor.with_input(&[]);

        let proof_nonce: [u32; PROOF_NONCE_NUM_WORDS] = [0; PROOF_NONCE_NUM_WORDS];
        let opts = SP1CoreOpts::default();

        while !executor.is_done() {
            let trace_chunk = executor.execute_chunk().unwrap();
            let mut gas_vm =
                GasEstimatingVM::new(&trace_chunk, program.clone(), proof_nonce, opts.clone());
            let _ = gas_vm.execute().unwrap();
        }

        let mut public_values = SP1PublicValues::from(executor.public_values_stream().as_slice());
        let exit_code = public_values.read::<i8>();
        let ckb_vm_cycles = public_values.read::<u64>();

        println!("Exit code: {}", exit_code);
        println!("CKB-VM cycles: {}", ckb_vm_cycles);
        println!(
            "SP1 instruction executed: {:.2}M",
            executor.global_clk() as f64 / 1_000_000.0
        );
        // since there is no syscall used, we use very simple calculation.
        println!(
            "SP1 cycles: {:.2}M",
            (executor.global_clk() * 8) as f64 / 1_000_000.0
        );
        let hash = sha2::Sha256::digest(elf_bytes);
        println!("ELF SHA256: {}", hex::encode(hash));

        if exit_code != 0 {
            panic!("ckb-vm exit code is not 0");
        }
        if executor.exit_code() != 0 {
            panic!("sp1 exit code is not 0");
        }
        if ckb_vm_cycles != 994360 {
            panic!("ckb-vm cycles not matched");
        }

        return;
    }

    let client = ProverClient::from_env().await;
    let stdin = SP1Stdin::new();

    if args.execute {
        let (mut public_values, report) =
            client.execute(CKB_VM_INTERPRETER_ELF, stdin).await.unwrap();
        let exit_code = public_values.read::<i8>();
        let ckb_vm_cycles = public_values.read::<u64>();
        println!("Exit code: {}", exit_code);
        println!("CKB-VM cycles: {}", ckb_vm_cycles);
        println!(
            "SP1 instruction executed: {:.2}M",
            report.total_instruction_count() as f64 / 1_000_000.0
        );
        println!(
            "SP1 cycles: {:.2}M",
            (report.total_instruction_count() * 8) as f64 / 1_000_000.0
        );
        let elf_bytes: &[u8] = &CKB_VM_INTERPRETER_ELF;
        let hash = sha2::Sha256::digest(elf_bytes);
        println!("ELF SHA256: {}", hex::encode(hash));
        if exit_code != 0 {
            panic!("ckb-vm exit code is not 0");
        }
        if ckb_vm_cycles != 994360 {
            panic!("ckb-vm cycles not matched");
        }
    } else {
        let pk = client
            .setup(CKB_VM_INTERPRETER_ELF)
            .await
            .expect("setup failed");

        let proof = client
            .prove(&pk, stdin)
            .core()
            .await
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        client
            .verify(&proof, pk.verifying_key(), None)
            .expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
