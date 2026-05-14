def x : Nat := (fun (n : Nat) => n) zero
def y : Nat := (fun (n : Nat) => succ n) zero
#eval x
#eval y
