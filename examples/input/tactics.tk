-- Proving propositions with intro / assumption / apply.
theorem id_thm : (A : Type) -> A -> A := by
  intro A; intro x; assumption

theorem const_thm : (A : Type) -> (B : Type) -> A -> B -> A := by
  intro A; intro B; intro a; intro b; assumption

axiom imp : (A : Type) -> (B : Type) -> A -> B
theorem use_apply : (A : Type) -> (B : Type) -> A -> B := by
  intro A; intro B; intro a; apply imp; exact A; exact B; assumption

#print id_thm
#print const_thm
#print use_apply
