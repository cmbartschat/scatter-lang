factorial: {
  1 swap       // put 1 on stack as accumulator, counter on top
  [
    (dup)      // exit if counter is down to 0
    dup rot *  // multiply accumulator by current counter
    swap 1 -   // decrement counter
  ]
  drop         // remove the counter, leave result
}

5 factorial    // 120
