-- A polymorphic identity, plus const, applied at Nat.
def id : (A : Type) -> A -> A :=
  fun (A : Type) (x : A) => x

def const : (A : Type) -> (B : Type) -> A -> B -> A :=
  fun (A : Type) (B : Type) (a : A) (b : B) => a

def two : Nat := succ (succ zero)

#check id
#check id Nat two
#eval id Nat (succ two)
#eval const Nat Bool two true
