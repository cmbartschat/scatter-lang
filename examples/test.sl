start_suite: { // suite_name -- fail_count logs
  ~suite_name~
  0
  suite_name ": " join
}

end_suite: { // fail_count logs --
  "done" join print
  ! "Suite failed" assert
}

fail: { // fail_count, logs -- fail_count, logs
  ~fail_count logs~
  fail_count ++ logs
}

should_equal: { // fail_count, logs, actual, expected -- fail_count, logs
  ~fail_count logs actual expected~
  actual expected ==
  {
    () fail_count logs "pass " join
    (1)
      fail_count ++ 
      logs
        "fail: `" join
        expected join
        "` != `" join
        actual join
        "` " join
  }
}

should_equal2_first: {swap}

should_equal2: { // fail_count, logs, a1, e1, a2, e2 -- fail_count, logs
  == rot rot
  == &&
  true should_equal
}

should_equal3_first: {rot rot}
should_equal3_second: {swap}

should_equal3: { // fail_count, logs, a1, e1, a2, e2, a3, e3 -- fail_count, logs
  == rot rot
  == && rot rot
  == &&
  true should_equal
}
