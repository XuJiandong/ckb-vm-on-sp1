//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use sp1_sdk::{include_elf, Elf, ProveRequest, Prover, ProverClient, ProvingKey, SP1Stdin};

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
}

#[tokio::main]
async fn main() {
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
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
