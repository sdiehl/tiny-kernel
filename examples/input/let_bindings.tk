-- let-bindings inside definitions.
def n : Nat :=
  let a : Nat := zero;
  succ (succ a)

def m : Nat :=
  let f : Nat -> Nat := fun (x : Nat) => succ x;
  f (f (f zero))

#eval n
#eval m
