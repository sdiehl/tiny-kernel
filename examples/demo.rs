use tiny_kernel::run_program;

fn main() {
    let src = "\
def two : Nat := succ (succ zero)\n\
def id : (A : Type) -> A -> A := fun (A : Type) (x : A) => x\n\
#check id Nat two\n\
#eval id Nat (succ two)\n\
";
    match run_program(src) {
        Ok(lines) => {
            for l in lines {
                println!("{l}");
            }
        }
        Err(e) => eprintln!("error: {e}"),
    }
}
