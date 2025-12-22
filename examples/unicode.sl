#!/usr/bin/env stacklang

#* "./test.sl"

"length" start_suite
"a" length 1 should_equal
"é" length 1 should_equal
"̃" length 1 should_equal
"h̃" length 2 should_equal
"✅" length 1 should_equal
"✅✅" length 2 should_equal
end_suite

"join" start_suite
"✅" dup join "✅✅" should_equal
"h" "̃" join "h̃" should_equal
end_suite

"substring" start_suite
"h̃" 1 2 substring "̃" should_equal
"h̃" 1 3 substring "̃" should_equal
"h̃x" 1 3 substring "̃x" should_equal
end_suite

"from_char" start_suite
771 from_char "̃" should_equal
9989 from_char "✅" should_equal
end_suite

"to_char" start_suite
"̃" to_char 771 should_equal
"✅" to_char 9989 should_equal
end_suite

"all passed." print
