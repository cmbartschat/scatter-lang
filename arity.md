# Arity

## What operations are possible

Create values
Move values around
Store values in variables
Read values from variables
Call functions

## What we need to be able to do

1. Record the types of inputs/outputs of intrinsics
2. Record the original positions in inputs of variables (generics)
3. Combine arities in serial
4. Combine arities in parallel
5. Resolve input types based on what is done with the value later, including through variables

## Observations

1. A value can be based on two different variable bindings using branches
2. A value can be based on a generic or variable binding using branches
3. Branches can sometimes define a variable, sometimes not.
4. Variables only apply to function scope
5. Blocks may need to be processed out of "order" sometimes, like in branches
```
fn: {
  ~v1~
  {
    () 
    




```
6. Resolving a resultant type should backfill to all possible bases - variables and so on. This can propagate further, for example:
```
fn: {   
        // - 
  dup   // 0 - 0 0
  ~v1~  // 0 - 0, v1=0
  drop  // 0, v1=0
  v1    // 0 - 0, v1=0
  1     // 0 - 0 n, v1=0
  +     // n - n, v1=n
}
```
