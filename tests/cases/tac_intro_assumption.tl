theorem id : (A : Type) -> A -> A := by intro A; intro x; assumption
theorem const : (A : Type) -> (B : Type) -> A -> B -> A := by
  intro A; intro B; intro a; intro b; assumption
#print id
#print const
