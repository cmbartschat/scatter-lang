# debug "./debug.sl"

countdown: { // count fn --
  [
    (over)
    over over eval
    swap
    --
    swap
  ]
  drop
  drop
}


fb: {
  {
    (dup 5 % ! over 3 % ! &&) drop "FizzBuzz"
    (dup 5 % !) drop "Buzz"
    (dup 3 % !) drop "Fizz"
  }
  print
}

20 @fb countdown
