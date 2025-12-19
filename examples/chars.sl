countdown: { // count fn --
  [
    (over 0 < !)
    over over eval
    swap
    --
    swap
  ]
  drop
  drop
}

between: { // v min max - b
  rot dup // min max v v
  rot // min v v max
  <  // min v b
  rot rot // b min v
  > !
  &&
}

digit_to_hex: { // n - s
  dup 0 16 between "between 0 and 15" assert
  {
    (dup 10 <)
      '0' to_char + from_char
    (1)
      10 -
      'a' to_char + from_char
  }
}

upper_hex: { // n - n
  16 / dup 1 % -
}

byte_to_hex: { // n - s
  dup
  upper_hex
  digit_to_hex
  swap
  16 % digit_to_hex
  join
}

print_character_with_code: { // n -
   dup
  byte_to_hex
  ": " join
  swap
  from_char join print
}

255 @print_character_with_code countdown
