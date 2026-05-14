def id : (A : Type) -> A -> A := fun (A : Type) (x : A) => x
#check id
#check id Nat zero
#eval id Nat (succ zero)
