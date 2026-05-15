# tiny-kernel

my hacky attempt at a dependent type theory kernel in Rust modeled on Lean. About 1800 lines. Don't use this for anything, it's just for fun.

```bash
cargo build
cargo test
```

Examples live as `.tk` source under `examples/input/` with one-line runners in `examples/`:

```bash
cargo run --example basics
cargo run --example polymorphic
cargo run --example let_bindings
cargo run --example universes
cargo run --example tactics
cargo run --example equality
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
