# optimize-ckb-vm

## Task
Your task is to optimize ckb-vm located at `deps/ckb-vm`. The performance benchmark uses the following commands:

Build:

```
cd program && cargo prove build
```

Run benchmark:

```
cargo run --release -- --minimal-execute
```

Current output:

```
Running `target/release/ckb-vm-interpreter --minimal-execute`
Exit code: 0
CKB-VM cycles: 994360
SP1 instruction executed: 27.05M
SP1 cycles: 216.43M
ELF SHA256: ff7758c06f581907c046963792c4be3494fbe260e5fadc40cbb89033a35c5f46
```

Our target is to reduce total SP1 cycles (ignore CKB-VM cycles). You can try different strategies. Only keep results with more than 2% improvement. Before updating code, record old SP1 cycles for further comparison.

If any performance improvement is found, commit it to git immediately (at new branch name: `opt` in ckb-vm). Don't mix multiple enhancements in one commit.

After updating code, check binary hash is changed(output as `ELF SHA256: xxx`). Report issue if this doesn't change.

## Hints

You can use memory for performance enhancement. It's not a memory-limited system, e.g., 100–500 MB of memory is not a problem.

You can also go through the `sp1` documentation or crates for more performance hints.
