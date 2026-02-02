use sp1_sdk::{include_elf, Elf, HashableKey, Prover, ProverClient, ProvingKey};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FIBONACCI_ELF: Elf = include_elf!("fibonacci-program");

#[tokio::main]
async fn main() {
    let prover = ProverClient::builder().cpu().build().await;
    let pk = prover.setup(FIBONACCI_ELF).await.expect("setup failed");
    println!("{}", pk.verifying_key().bytes32());
}
