fibonacci: {
  0 1
  [
    (rot dup) // check if the counter is down to 0
    1 -       // subtract one from the counter
    rot rot   // put the counter back at the bottom of the stack
    dup rot + // get the next number in the sequence
  ]
  drop drop   // leave the stack with just the result
}

20 fibonacci // 6765
