//! ## Usage
//! ```shell
//! # Fast execution with gas estimation (recommended for development)
//! cargo run --release -- --minimal-execute --mode vm
//! cargo run --release -- --minimal-execute --mode native
//!
//! # Full execution via ProverClient
//! RUST_LOG=info cargo run --release -- --execute --mode vm
//! RUST_LOG=info cargo run --release -- --execute --mode native
//!
//! # Generate proof (requires significant resources)
//! RUST_LOG=info cargo run --release -- --prove --mode vm
//! ```

use clap::{Parser, ValueEnum};
use sha2::Digest;
use sp1_core_executor::{GasEstimatingVM, MinimalExecutor, Program, SP1CoreOpts};
use sp1_hypercube::air::PROOF_NONCE_NUM_WORDS;
use sp1_sdk::{
    include_elf, Elf, ProveRequest, Prover, ProverClient, ProvingKey, SP1PublicValues, SP1Stdin,
};
use std::sync::Arc;

/// The ELF for CKB-VM interpreter (runs k256_ecdsa inside CKB-VM)
pub const CKB_VM_INTERPRETER_ELF: Elf = include_elf!("ckb-vm-interpreter-program");

/// The ELF for native k256_ecdsa (runs directly on SP1)
pub const NATIVE_K256_ECDSA_ELF: &[u8] = include_bytes!("../../binaries/k256_ecdsa_sp1");

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum Mode {
    /// Run k256_ecdsa natively on SP1
    Native,
    /// Run k256_ecdsa inside CKB-VM interpreter on SP1
    #[default]
    Vm,
}

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

    #[arg(long, value_enum, default_value_t = Mode::Vm)]
    mode: Mode,
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

    let (elf_bytes, mode_desc): (&[u8], &str) = match args.mode {
        Mode::Native => (
            NATIVE_K256_ECDSA_ELF,
            "native (k256_ecdsa runs directly on SP1)",
        ),
        Mode::Vm => (
            &CKB_VM_INTERPRETER_ELF,
            "vm (k256_ecdsa runs inside CKB-VM on SP1)",
        ),
    };

    if args.minimal_execute {
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

        println!("Mode: {}", mode_desc);
        println!("Exit code: {}", exit_code);
        if matches!(args.mode, Mode::Vm) {
            let ckb_vm_cycles = public_values.read::<u64>();
            println!("CKB-VM cycles: {}", ckb_vm_cycles);
        }
        println!(
            "SP1 instruction executed: {:.2}M",
            executor.global_clk() as f64 / 1_000_000.0
        );
        println!(
            "SP1 cycles: {:.2}M",
            (executor.global_clk() * 8) as f64 / 1_000_000.0
        );
        let hash = sha2::Sha256::digest(elf_bytes);
        println!("ELF SHA256: {}", hex::encode(hash));

        if exit_code != 0 {
            panic!("exit code is not 0");
        }
        if executor.exit_code() != 0 {
            panic!("sp1 exit code is not 0");
        }

        return;
    }

    let client = ProverClient::from_env().await;
    let stdin = SP1Stdin::new();

    if args.execute {
        let (mut public_values, report) =
            client.execute(Elf::Static(elf_bytes), stdin).await.unwrap();
        let exit_code = public_values.read::<i8>();

        println!("Mode: {}", mode_desc);
        println!("Exit code: {}", exit_code);
        if matches!(args.mode, Mode::Vm) {
            let ckb_vm_cycles = public_values.read::<u64>();
            println!("CKB-VM cycles: {}", ckb_vm_cycles);
        }
        println!(
            "SP1 instruction executed: {:.2}M",
            report.total_instruction_count() as f64 / 1_000_000.0
        );
        println!(
            "SP1 cycles: {:.2}M",
            (report.total_instruction_count() * 8) as f64 / 1_000_000.0
        );
        let hash = sha2::Sha256::digest(elf_bytes);
        println!("ELF SHA256: {}", hex::encode(hash));
        if exit_code != 0 {
            panic!("exit code is not 0");
        }
    } else {
        println!("Mode: {}", mode_desc);
        let pk = client
            .setup(Elf::Static(elf_bytes))
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
