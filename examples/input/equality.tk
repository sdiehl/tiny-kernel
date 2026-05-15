-- Propositional equality with refl / rfl.
theorem zero_eq : Eq Nat zero zero := by refl
theorem one_eq  : Eq Nat (succ zero) (succ zero) := by rfl

theorem id_eq : (A : Type) -> (a : A) -> Eq A a a := by
  intro A; intro a; refl

#print zero_eq
#print one_eq
#print id_eq
