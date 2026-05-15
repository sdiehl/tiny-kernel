//! Runs every `.tk` source under `examples/input/`.
//! Pass a filename (e.g. `cargo run --example run_all -- tactics`) to run just one.

use std::path::PathBuf;
use tiny_kernel::run_program;

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/input");
    let filter = std::env::args().nth(1);

    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("tk"))
        .filter(|p| {
            filter.as_ref().is_none_or(|f| {
                p.file_stem().and_then(|s| s.to_str()) == Some(f.as_str())
                    || p.file_name().and_then(|s| s.to_str()) == Some(f.as_str())
            })
        })
        .collect();
    paths.sort();

    if paths.is_empty() {
        eprintln!("no .tk files matched in {}", dir.display());
        std::process::exit(1);
    }

    for (i, path) in paths.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!("==> {}", path.file_name().unwrap().to_string_lossy());
        let src = std::fs::read_to_string(path).expect("read input");
        match run_program(&src) {
            Ok(lines) => lines.iter().for_each(|l| println!("{l}")),
            Err(e) => {
                eprintln!("error in {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    }
}
