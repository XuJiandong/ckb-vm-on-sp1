//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use ckb_vm::cost_model::estimate_cycles;
use ckb_vm::{Bytes, DefaultMachineRunner, SupportMachine, Syscalls};

const CODE: &[u8] = include_bytes!("secp256k1_ecdsa_ckbvm");

pub struct DebugSyscall {}

impl<Mac: SupportMachine> Syscalls<Mac> for DebugSyscall {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), ckb_vm::error::Error> {
        Ok(())
    }

    fn ecall(&mut self, _machine: &mut Mac) -> Result<bool, ckb_vm::error::Error> {
        Ok(true)
    }
}

fn main_interpreter64(code: Bytes, args: Vec<Bytes>) {
    let core_machine = ckb_vm::DefaultCoreMachine::<u64, ckb_vm::SparseMemory<u64>>::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_A | ckb_vm::ISA_MOP,
        ckb_vm::machine::VERSION2,
        u64::MAX,
    );
    let machine_builder = ckb_vm::RustDefaultMachineBuilder::new(core_machine)
        .instruction_cycle_func(Box::new(estimate_cycles));
    let mut machine = machine_builder.syscall(Box::new(DebugSyscall {})).build();
    machine
        .load_program(&code, args.into_iter().map(Ok))
        .expect("load program");
    machine.run().expect("run program");
}

fn main() {
    main_interpreter64(CODE.into(), vec![])
}
