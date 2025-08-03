sign: {
  {
    (dup 0 >) "positive"
    (dup 0 <) "negative"
    (dup 0 ==) "zero"
    (1) "NaN"
  }
  swap drop
}
4 sign
-1 sign
-1  0.5 ** sign
