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
use sp1_core_executor::{GasEstimatingVM, MinimalExecutor, Program, SP1CoreOpts};
use sp1_hypercube::air::PROOF_NONCE_NUM_WORDS;
use sp1_sdk::{include_elf, Elf, ProveRequest, Prover, ProverClient, ProvingKey, SP1Stdin};
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

        let mut total_gas: u64 = 0;

        while !executor.is_done() {
            let trace_chunk = executor.execute_chunk().unwrap();
            let mut gas_vm =
                GasEstimatingVM::new(&trace_chunk, program.clone(), proof_nonce, opts.clone());
            let report = gas_vm.execute().unwrap();
            if let Some(gas) = report.gas() {
                total_gas += gas;
            }
        }

        println!(
            "Cycles executed: {:.1}M",
            executor.global_clk() as f64 / 1_000_000.0
        );
        println!("Total gas cost: {:.1}M", total_gas as f64 / 1_000_000.0);

        let exit_code = executor.exit_code();
        if exit_code != 0 {
            panic!("Execution failed with exit code: {}", exit_code);
        }
        return;
    }

    let client = ProverClient::from_env().await;
    let stdin = SP1Stdin::new();

    if args.execute {
        let (mut public_values, report) =
            client.execute(CKB_VM_INTERPRETER_ELF, stdin).await.unwrap();
        let exit_code = public_values.read::<i8>();
        println!("Program executed successfully. Exit code = {}", exit_code);
        println!("Number of cycles: {}", report.total_instruction_count());
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
