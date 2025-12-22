#!/usr/bin/env stacklang
// StackLang Intrinsics Unit Tests

#* "./test.sl"

// Arithmetic Operations
"+" start_suite
3 4 + 7 should_equal
-5 3 + -2 should_equal
0 0 + 0 should_equal
3.14 2.86 + 6 should_equal
end_suite

"-" start_suite
7 3 - 4 should_equal
3 7 - -4 should_equal
0 5 - -5 should_equal
10.5 0.5 - 10 should_equal
end_suite

"*" start_suite
3 4 * 12 should_equal
-2 3 * -6 should_equal
0 5 * 0 should_equal
2.5 4 * 10 should_equal
end_suite

"/" start_suite
12 3 / 4 should_equal
10 4 / 2.5 should_equal
-6 2 / -3 should_equal
1 2 / 0.5 should_equal
end_suite

"%" start_suite
10 3 % 1 should_equal
7 2 % 1 should_equal
8 4 % 0 should_equal
-7 3 % -1 should_equal
end_suite

"**" start_suite
2 3 ** 8 should_equal
5 2 ** 25 should_equal
4 0.5 ** 2 should_equal
3 0 ** 1 should_equal
end_suite

"++" start_suite
5 ++ 6 should_equal
-1 ++ 0 should_equal
0 ++ 1 should_equal
3.5 ++ 4.5 should_equal
end_suite

"--" start_suite
5 -- 4 should_equal
0 -- -1 should_equal
-2 -- -3 should_equal
3.7 -- 2.7 should_equal
end_suite

// Comparison Operations
"==" start_suite
5 5 == true should_equal
3 4 == false should_equal
"hello" "hello" == true should_equal
"hi" "bye" == false should_equal
true true == true should_equal
false true == false should_equal
'a' "a" == true should_equal
'"' "\"" == true should_equal
'\'' "'" == true should_equal
'\n' "
" == true should_equal
'\0' '\x00' == true should_equal
'A' '\x41' == true should_equal
"" "\
" == true should_equal
end_suite

"<" start_suite
3 5 < true should_equal
5 3 < false should_equal
5 5 < false should_equal
-2 1 < true should_equal
end_suite

">" start_suite
5 3 > true should_equal
3 5 > false should_equal
5 5 > false should_equal
1 -2 > true should_equal
end_suite

// String Operations
"join" start_suite
"hello " "world" join "hello world" should_equal
1 2 3 join join "123" should_equal
true "/" false join join "true/false" should_equal
end_suite

"substring" start_suite
"hello world" 0 5 substring "hello" should_equal
"hello world" 6 11 substring "world" should_equal
"hello world" 6 15 substring "world" should_equal
"hello" 6 11 substring "" should_equal
"" 4 5 substring "" should_equal
"hello world" 4 3 substring "" should_equal
end_suite

"to_char" start_suite
"a" to_char 97 should_equal
"A" to_char 65 should_equal
end_suite

"from_char" start_suite
97 from_char "a" should_equal
65 from_char "A" should_equal
end_suite

"index" start_suite
"hello world" "h" index 0 should_equal
"hello world" "x" index -1 should_equal
"hello world" "w" index 6 should_equal
"hello world" "world" index 6 should_equal
"hello world" "worlds" index -1 should_equal
"" "" index 0 should_equal
"test" "" index 0  should_equal
"hi" "hello" index -1 should_equal
end_suite

"length" start_suite
"hello world" length 11 should_equal
"" length 0 should_equal
end_suite

// Boolean Operations
"&&" start_suite
true true && true should_equal
true false && false should_equal
false true && false should_equal
false false && false should_equal
3 4 && 4 should_equal
0 1 && 0 should_equal
"hi" "" && "" should_equal
"" 0 && "" should_equal
end_suite

"||" start_suite
true true || true should_equal
true false || true should_equal
false true || true should_equal
false false || false should_equal
3 4 || 3 should_equal
0 1 || 1 should_equal
"hi" "" || "hi" should_equal
"" 0 || 0 should_equal
end_suite

"!" start_suite
true ! false should_equal
false ! true should_equal
0 ! true should_equal
1 ! false should_equal
"" ! true should_equal
"hello" ! false should_equal
end_suite

// Stack Manipulation
"dup" start_suite
5 dup 5 swap 5 should_equal2
end_suite

"swap" start_suite
1 2 swap 2 swap 1 should_equal2
end_suite

"over" start_suite
1 2 over 1 should_equal3_first 2 should_equal3_second 1 should_equal3
end_suite

"rot" start_suite
1 2 3 rot 2 should_equal3_first 3 should_equal3_second 1 should_equal3
end_suite

"drop" start_suite
1 2 drop 1 should_equal
end_suite

// Function Definition and Call
"function_definition_and_call" start_suite
square: {dup *}
5 square 25 should_equal

add_ten: {10 +}
7 add_ten 17 should_equal

double: {2 *}
4 double 8 should_equal
end_suite

// Complex Function
"distance_function" start_suite
sqrt: {0.5 **}
distance: {
  rot - square
  rot rot
  - square
  +
  sqrt
}
3 0 0 4 distance 5 should_equal
end_suite

// Branch
"simple_branch" start_suite
check_even: {
  {
    (2 %) "odd"
    (1) "even"
  }
}
3 check_even "odd" should_equal
4 check_even "even" should_equal
end_suite

"grade_branch" start_suite
grade: {
  {
    (dup 60 <) "F"
    (dup 70 <) "D"
    (dup 80 <) "C"
    (dup 90 <) "B"
    (dup 100 <) "A"
    (1) "A+"
  }
  swap drop
}
45 grade "F" should_equal
65 grade "D" should_equal
75 grade "C" should_equal
85 grade "B" should_equal
95 grade "A" should_equal
105 grade "A+" should_equal
end_suite

"sign_function" start_suite
sign: {
  {
    (dup 0 >) "positive"
    (dup 0 <) "negative"
    (dup 0 ==) "zero"
    (1) "NaN"
  }
  swap drop
}
4 sign "positive" should_equal
-1 sign "negative" should_equal
0 sign "zero" should_equal
end_suite

// Loop
"countdown_loop" start_suite
countdown: {
  [
    (dup)
    dup
    --
  ]
}
2 countdown 2 should_equal3_first 1 should_equal3_second 0 should_equal3
end_suite

"factorial_loop" start_suite
factorial: {
  1 swap
  [
    (dup)
    dup rot *
    swap 1 -
  ]
  drop
}
5 factorial 120 should_equal
4 factorial 24 should_equal
3 factorial 6 should_equal
1 factorial 1 should_equal
0 factorial 1 should_equal
end_suite

"fibonacci_loop" start_suite
fibonacci: {
  0 1
  [
    (rot dup)
    1 -
    rot rot
    dup rot +
  ]
  drop drop
}
0 fibonacci 0 should_equal
1 fibonacci 1 should_equal
5 fibonacci 5 should_equal
10 fibonacci 55 should_equal
end_suite

// Type Conversion and Truthiness
"truthiness" start_suite
false ! true should_equal
0 ! true should_equal
"" ! true should_equal
1 ! false should_equal
"hello" ! false should_equal
-1 ! false should_equal
end_suite

// Complex Expression
"complex_expressions" start_suite
5 3 + 2 * 16 should_equal
10 dup / 1 should_equal
1 2 3 rot + - -2 should_equal
10 5 < false should_equal
7 7 - 5 || 5 should_equal
10 20 over - == true should_equal
end_suite

// String Literal Tests
"string_literals" start_suite
"hello" "hello" == true should_equal
"" "" == true should_equal
"test" "different" == false should_equal
end_suite

// Number Literal Tests
"number_literals" start_suite
42 42 == true should_equal
3.14 3.14 == true should_equal
-17 -17 == true should_equal
0 0 == true should_equal
end_suite

// Boolean Literal Tests
"boolean_literals" start_suite
true true == true should_equal
false false == true should_equal
true false == false should_equal
end_suite

// Single function Tests
constant: 300
double_increment: ++ ++
noop:
"single_functions" start_suite
constant 300 should_equal
3 double_increment 5 should_equal
noop
end_suite

// Function address
"eval" start_suite
1 1 @+ eval 2 should_equal
@double_increment 3 swap eval 5 should_equal
end_suite

"multiline" start_suite
/* 0 "example" assert */
0
/* 5555 + */ 1 +
/**/ 2 +
/* 5555 + / */ 
/* 5555 + /* */ 
/* 5555 + /* **/ 
//* something
4 + 
7 should_equal
multiline_comment_single: /* something 
*/ 3
multiline_comment_single multiline_comment_single * 9 should_equal
"/*hello*/" length 9 should_equal
end_suite

"all passed." print
