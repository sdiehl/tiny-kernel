use tiny_kernel::run_program;

fn main() {
    let src = include_str!("input/tactics.tk");
    match run_program(src) {
        Ok(lines) => lines.iter().for_each(|l| println!("{l}")),
        Err(e) => eprintln!("error: {e}"),
    }
}
