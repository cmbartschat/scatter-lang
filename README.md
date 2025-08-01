```
start end -> swap
end start -> dup
end start start -> rot
start start end


end next
next end
next end next
next end next end

add: {
  1 1 +
}

fib: d => d {
  0 1
  // iterations, current, next
  // swap -> current, next, iterations
  // dup -> current, next, iterations, iterations
  // iterations, current, next iterations
  {(rot dup)
    1 -
    rot rot
    dup rot +
  }
  drop
  swap
  drop
}


_fb: {
  {
    (dup 5 % ! over 3 % ! && !) drop "FizzBuzz" print
    (dup 5 % !) drop "Fizz" print
    (dup 3 % !) drop "Buzz" print
    (1) print
  }
}

fb: {
  [
    (0 >=)
    1 -
    dup _fb
  ]
}


rfib: d => d {
  dup 1 > {
    1 - dup rfib swap 1 - rfib +
  }
}

// < >

// { } - if

// ( / ) - while
```

// do while: block -> condition -> reset
// if: condition -> block
// while: condition -> block -> reset
// switch, if/else
