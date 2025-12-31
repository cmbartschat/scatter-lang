#!/usr/bin/env stacklang

"LENGTH" print

check_length: { // string length --
  over length == swap assert
}

"😀" 1 check_length
"a😀b" 3 check_length
"👍🏽" 2 check_length
"🇫🇷" 2 check_length
"e\u0301" 2 check_length
"𝄞" 1 check_length
"👨‍👩‍👧‍👦" 7 check_length
"क्‍ष" 4 check_length
"中文🙂" 3 check_length
"😃😄😁" 3 check_length
"a𐍈b" 3 check_length
"Z͑͗͂" 4 check_length
"नमस्ते" 6 check_length
"👩‍❤️‍💋‍👨" 8 check_length
"𠜎𠜱𠝹" 3 check_length

"INDEX" print

check_char_index: { // string index expected --
  rot rot // expected string index
  over swap dup 1 + substring // expected string actual
  rot == swap "index" join assert
}

"😀" 0 "😀" check_char_index
"a😀b" 1 "😀" check_char_index
"👍🏽" 1 "🏽" check_char_index
"🇫🇷" 0 "🇫" check_char_index
// come back when we can do unicode escapes "é" 1 "́" check_char_index
"𝄞" 0 "𝄞" check_char_index
"👨‍👩‍👧‍👦" 0 "👨" check_char_index
"क्‍ष" 3 "ष" check_char_index
"中文🙂" 1 "文" check_char_index
"😃😄😁" 2 "😁" check_char_index
"a𐍈b" 1 "𐍈" check_char_index
"Z͑͗͂" 3 "͂" check_char_index
"नमस्ते" 3 "्" check_char_index
"👩‍❤️‍💋‍👨" 7 "👨" check_char_index
"𠜎𠜱𠝹" 2 "𠝹" check_char_index

"TO/FROM_CHAR" print

check_conversion: { // string value --
  over over
  swap to_char == "to_char" assert
  from_char == "from_char" assert
}


"😀" 128512 check_conversion
"👍" 128077 check_conversion
"🏽" 127997 check_conversion
"🇫" 127467 check_conversion
"🇷" 127479 check_conversion
"𝄞" 119070 check_conversion
"𐍈" 66376 check_conversion
"中" 20013 check_conversion
"文" 25991 check_conversion
"क" 2325 check_conversion
"्" 2381 check_conversion
"ष" 2359 check_conversion
"́" 769 check_conversion
"‍" 8205 check_conversion
"👨" 128104 check_conversion
"👩" 128105 check_conversion
"❤" 10084 check_conversion
"💋" 128139 check_conversion
"𠜎" 132878 check_conversion
"𠝹" 132985 check_conversion

"INDEX:" print

check_index: { // haystack needle index
  rot  // needle index haystack
  rot
  index
  == "index" assert
}

"😀hi" "hi" 1 check_index
"a😀hi" "hi" 2 check_index
"😀😀hi" "hi" 2 check_index
"👍hi" "hi" 1 check_index
"👍🏽hi" "hi" 2 check_index
"hi😀" "😀" 2 check_index
"hi👍" "👍" 2 check_index
"hi👍🏽" "👍🏽" 2 check_index
"😀a😀b" "😀b" 2 check_index
"🇫🇷hi" "hi" 2 check_index
"a🇫🇷hi" "hi" 3 check_index
"👨‍👩‍👧‍👦hi" "hi" 7 check_index
"𝄞hi" "hi" 1 check_index
"中😀文" "😀文" 1 check_index
"e\u0301hi" "hi" 2 check_index

"SUBSTRING" print

check_substring: {
  substring == "substring" assert
}

"😀b" "😀a😀b" 2 4 check_substring
"hi" "😀hi" 1 3 check_substring
"👍" "a👍b" 1 2 check_substring
"👍🏽" "a👍🏽b" 1 3 check_substring
"🇫🇷" "x🇫🇷y" 1 3 check_substring
"🇫" "x🇫🇷y" 1 2 check_substring
"b" "😀b" 1 2 check_substring
"😀" "😀😀" 0 1 check_substring
"😀" "a😀b" 1 2 check_substring
"👨‍👩‍👧‍👦" "👨‍👩‍👧‍👦hi" 0 7 check_substring
"hi" "👨‍👩‍👧‍👦hi" 7 9 check_substring
"文😀" "中文😀a" 1 3 check_substring
"e\u0301" "ae\u{301}b" 1 3 check_substring
"́" "e\u0301" 1 2 check_substring
"𐍈" "a𐍈b" 1 2 check_substring
"𠝹" "𠜎𠝹𠜱" 1 2 check_substring

"SUCCESS" print
