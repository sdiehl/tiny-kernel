axiom imp : (A : Type) -> (B : Type) -> A -> B
theorem t : (A : Type) -> (B : Type) -> A -> B := by
  intro A; intro B; intro a; apply imp; exact A; exact B; assumption
#print t
