def id : (A : Type) -> A -> A := fun A x => x
def two : Nat := succ (succ zero)
def apply_id : Nat := id _ two
#check apply_id
#eval apply_id
