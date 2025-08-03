grade: {
  {
    (dup 60 <) "F"
    (dup 70 <) "D"
    (dup 80 <) "C"
    (dup 90 <) "B"
    (dup 100 <) "A"
    (1) "A+"
  }
  swap drop
}

99 grade // "A"
83 grade // "B"
