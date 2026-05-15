# tiny-kernel

my hacky attempt at a dependent type theory kernel in Rust modeled on Lean. About 1800 lines. Don't use this for anything, it's just for fun.

```bash
cargo build
cargo test
```

Examples live as `.tk` source under `examples/input/`. A single runner globs them all (or pass a name to run one):

```bash
cargo run --example run_all
cargo run --example run_all -- tactics
```

The same files are snapshotted in `tests/snapshots.rs` (`cargo insta review` after editing them).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
