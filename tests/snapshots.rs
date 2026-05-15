use tiny_kernel::run_program;

fn run(path: &std::path::Path) -> String {
    let src = std::fs::read_to_string(path).unwrap();
    match run_program(&src) {
        Ok(lines) => lines.join("\n"),
        Err(e) => format!("error: {e}"),
    }
}

#[test]
fn cases() {
    insta::glob!("cases/*.tl", |path| {
        insta::assert_snapshot!(run(path));
    });
}

#[test]
fn examples() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    insta::glob!(root, "examples/input/*.tk", |path| {
        insta::assert_snapshot!(run(path));
    });
}
