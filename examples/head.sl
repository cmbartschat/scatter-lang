#!/usr/bin/env stacklang
15 // number of lines to read
[
  (readline)
  swap
  {
    (dup 0 >) -- swap print
    (1) swap drop
  }
]
drop
drop
