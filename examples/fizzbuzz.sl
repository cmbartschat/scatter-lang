_fb: {
  {
    (dup 5 % ! over 3 % ! &&) drop "FizzBuzz"
    (dup 5 % !) drop "Buzz"
    (dup 3 % !) drop "Fizz"
  }
}

fb: {
  [
    (dup 0 >)
    dup _fb
    swap
    1 -
  ]
}

20 fb
