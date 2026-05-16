# tiny-kernel

my hacky attempt at a dependent type theory kernel in Rust modeled on Lean. About 1500 lines. Don't use this for anything, it's just for fun.

```bash
cargo build
cargo test
```

Examples are `.tl` files under [`examples/input/`](examples/input/). Run with:

```bash
cargo run --example run_all
cargo run --example run_all -- tactics
```

Also has insta snapshots as a reference of expected outputs.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
