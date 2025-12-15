_fb: {
  {
    (dup 5 % ! over 3 % ! &&) drop "FizzBuzz"
    (dup 5 % !) drop "Buzz"
    (dup 3 % !) drop "Fizz"
  }
  print
}

fb: {
  [
    (dup 0 >)
    dup _fb
    1 -
  ]
  drop
}

20 fb
