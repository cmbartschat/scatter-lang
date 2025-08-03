year_to_generation: {
  {
    (dup 2010 >) "Gen Alpha"
    (dup 1996 >) "Gen Z"
    (dup 1980 >) "Millennial"
    (dup 1964 >) "Gen X"
    (dup 1945 >) "Baby Boomer"
    (1) "Something before boomer"
  }
  swap drop
}

1996 year_to_generation // "Millennial"
