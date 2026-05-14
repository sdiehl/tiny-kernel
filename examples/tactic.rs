use tiny_kernel::run_program;

fn main() {
    let src = "\
theorem id_eq : (A : Type) -> (a : A) -> Eq A a a := by\n\
  intro A; intro a; refl\n\
\n\
theorem id_id : (A : Type) -> A -> A := by\n\
  intro A; intro x; assumption\n\
\n\
#check id_eq\n\
#check id_id\n\
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
