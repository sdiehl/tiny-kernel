-- Definitions, application, and #eval / #check.
def two : Nat := succ (succ zero)
def three : Nat := succ two
def four : Nat := succ three

#check two
#check three
#eval three
#eval four
