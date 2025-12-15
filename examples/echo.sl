echo: {
  [
    "say something:" print
    readline
    {
      (dup)
        swap
        "you typed: " swap join
        "!" join print
      (1) swap drop
    }
    ()
  ]
}

echo
