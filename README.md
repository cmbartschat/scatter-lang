# Stacklang

A stack based programming language.

## Quick Start

```
// Literals push values onto the stack

42 true "hi"    // [42, true, "hi"]

// Operations work in reverse polish notation - taking values from the stack and placing the results back on

3             // [3]
4             // [3, 4]
5             // [3, 4, 5]
+             // [3, 9]
+             // [12]

// Functions are defined with `name: {body}` and called by name

square: {dup *}
5 square        // [25]

// Branches use conditions to choose which block to execute

sign: {
  {
    (dup 0 >) "positive"
    (dup 0 <) "negative"
    (dup 0 ==) "zero"
    (1) "NaN"
  }
  swap drop     // Drop the passed in value
}

4 sign          // ["positive"]
-1 sign         // ["negative"]
-1 0.5 ** sign  // ["NaN"]

// Loops re-run a block until an exit condition is reached

countdown: {
  [
    (dup)
    dup
    --
  ]
}

5 countdown     // [5, 4, 3, 2, 1]
```

## Stack

The stack is the core data structure in Stacklang. All values exist on the stack, and all operations manipulate the stack. There are no variables, all state is on the stack.

The stack grows as items are pushed - newer items are pushed to the end:

```
3               // [3]
4               // [3, 4]
5               // [3, 4, 5]
```

Stack operations consume their operands:

```
5 3             // [5, 3]
-               // [2] - removes 5 and 3 from stack, leaves 2
```

Functions operate on the stack, taking arguments from the stack as needed:

```
double: {
  2 *
}
4 double        // [8]
```

## Literals

Literals push values directly onto the stack:

```
// Numbers (doubles)

42
3.14
-17

// Booleans (true/false)

true
false

// Strings

"hello"
"world!"
""
"\""            // Escaped quote character
```

## Types

Each value on the stack is either a number, string, or boolean. Some intrinsics accept any type, but arithmetic operations will exit with an error if they are not used on numbers.

Values can be converted to booleans either by conditions or by boolean operators, in which case `false`, `0`, `NaN`, and `""` are considered falsy (equivalent to false), and all other values are considered truthy (equivalent to true).

## Intrinsics

Intrinsics are built-in operations predefined in the language.

### Arithmetic Operations

```
+               // Addition: a b -> (a + b)
-               // Subtraction: a b -> (a - b)
*               // Multiplication: a b -> (a * b)
/               // Division: a b -> (a / b)
%               // Modulo: a b -> (a % b)
**              // Exponentiation: a b -> (a ** b)
--              // Increment: a b -> (a - 1)
++              // Decrement: a b -> (a + 1)
```

### Comparison Operations

```
==              // Equal: a b -> (a == b)
<               // Less than: a b -> (a < b)
>               // Greater than: a b -> (a > b)
```

### Boolean Operations

```
&&              // Logical AND: a b -> (a && b)
||              // Logical OR: a b -> (a || b)
!               // Logical NOT: a -> (!a)
```

Note that in the case of `||` and `&&`, types are preserved:

```
3 4 ||          // [3]
0 1 ||          // [1]
"hi" "" ||      // ["hi"]
"" 0 ||         // [0]

3 4 &&          // [4]
0 1 &&          // [0]
"hi" "" &&      // [""]
"" 0 &&         // [""]
```

### Stack Manipulation

```
dup             // Duplicate top value: a -> a a
swap            // Swap top two values: a b -> b a
over            // Copy second value to top: a b -> a b a
rot             // Rotate top three values: a b c -> b c a
drop            // Remove top value: a b -> a
```

### Output

```
print           // Print the top value to the screen
```

### Examples

```
10 5 <          // [false]
5 3 + 2 *       // [16]
10 dup /        // [1]
1 2 3 rot + -   // [-2]
1 1 + drop      // []
7 7 - 5 ||      // [5]
10 20 over - == // [true]
```

## Functions

Functions are named blocks of code that operate on the stack. They have no explicit parameters - they work with whatever is on the stack when called. To return a value, functions place the result on the stack for the caller to use.

### Syntax

```
name: {
  ...
}
```

### Examples

```
square: {dup *}
5 square        // [25]

add_ten: {10 +}
7 add_ten       // [17]

greet: {"Hello" print}
greet           // [], "Hello" printed
```

Functions can take any number of arguments, based on what is already in the stack. If there are insufficient values on the stack, an error will be thrown.

```
square: {dup *}

sqrt: {0.5 **}

// x1 y1 x2 y2 -> distance
distance: {
  rot - square  // evaluate (y2 - y1) ^ 2
  rot rot       // bring x1 and x2 to the top of the stack
  - square      // evaluate (x2 - x1) ^ 2
  +             // add the previous results
  sqrt
}

3 0             // [3, 0], define first point
0 4             // [3, 0, 0, 4], define second point
distance        // [5]
```

## Branches

Branching provides conditional execution. Branches evaluate conditions top-to-bottom and execute the first matching case.

### Syntax

```
{
  (condition1) action1
  (condition2) action2
}
```

### Condition Evaluation

Conditions contain an expression that is evaluated to make a control flow decision. If the contained expression evaluates to a truthy value, the associated action executes. The empty condition `()` evaluates what is currently on the top of stack and consumes it. `(1)` can be used as a fallback case because it always evaluates to true.

### Examples

```
check_even: {
  {
    (2 %) "Odd"
    (1) "Even"
  }
}

3 check_even    // ["Odd"]
4 check_even    // ["Even"]
```

```
grade: {
  {
    (dup 60 <) "F"
    (dup 70 <) "D"
    (dup 80 <) "C"
    (dup 90 <) "B"
    (dup 100 <) "A"
    (1) "A+"
  }
  swap drop     // remove the score, leaving just the grade
}

83 grade        // ["B"]
```

## Loops

Loops provide the ability to run code repeatedly. It consists of an optional pre-condition, a body, and an optional post-condition.

The pre-condition is checked at the start of each iteration, and the post-condition is checked after each iteration. If either condition returns a falsy value, the loop exits. A loop with no conditions will repeat indefinitely.

### Syntax

```
[(pre_condition) body (post_condition)]
```

### Loop Execution

1. Check pre-condition if it exists - if false, exit
2. Execute body
3. Check post-condition if it exists - if false, exit
4. Go back to step 1

### Examples

```
countdown: {
  [
    (dup)       // Check if the current value is positive, exit otherwise
    dup print   // Print the current value
    --          // Decrement the value
  ]
  drop          // Consume the value
}

10 countdown    // Prints numbers 10 down to 1
```

```
fibonacci: {
  0 1
  [
    (rot dup)   // Check if the counter is down to 0
    1 -         // Subtract one from the counter
    rot rot     // Put the counter back at the bottom of the stack
    dup rot +   // Get the next number in the sequence
  ]
  drop drop     // Leave the stack with just the result
}

20 fibonacci    // 6765
```

```
factorial: {
  1 swap        // Put 1 on stack as accumulator, counter on top
  [
    (dup)       // Exit if counter is down to 0
    dup rot *   // Multiply accumulator by current counter
    swap 1 -    // Decrement counter
  ]
  drop          // Remove the counter, leave result
}

5 factorial     // 120
```

## Comments

When // occurs outside a string, the remainder of the line is ignored.

```
1 // 2 +
```

The `2 +` is in the comment so it is ignored.
