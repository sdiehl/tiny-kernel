use tiny_kernel::run_program;

#[test]
fn cases() {
    insta::glob!("cases/*.tl", |path| {
        let src = std::fs::read_to_string(path).unwrap();
        let out = match run_program(&src) {
            Ok(lines) => lines.join("\n"),
            Err(e) => format!("error: {e}"),
        };
        insta::assert_snapshot!(out);
    });
}
