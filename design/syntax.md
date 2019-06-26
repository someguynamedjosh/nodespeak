# Syntax description

## Values

### Literals

Waveguide supports a handful of literals:

`1`, `0`, `-12_03` are all integers.

`0x31`, `0x5f`, `0xFF_FF` are all integers expressed in hex notation.

`0b110001101`, `0b1101_0100` are all integers expressed in binary notation.

`0o107`, `0o41`, `0o501` are all integers expressed in octal notation. The 
more standard octal notation, `0123`, will still be interpreted as octal, but 
produce a compiler warning. 

`1.0`, `.124`, `12e10`, `14.0e+43`, `9_8.76e-1_2` are all floating point values.
Note that there is no double type, only float.

In all the above examples, underscores serve as seperators that can be used
to make large numbers more readable. When the file is parsed, the number literal
is parsed as if the underscores were never inserted.

`true`, `false` are the two acceptable literals for boolean values. Other truthy
or falsey values such as `1` and `0` can be used where a boolean is needed, but
they will be cast to a boolean value beforehand. It is best to explicitly use
`true` or `false`.

Array literals can be specified using brackets: `[1, 2, 3]`.

Note that there is no string literal. Waveguide does not support dynamically
sized runtime content like strings, although they will likely be added in the 
future as a compile-time-only datatype.

### Variables

Variables can be referenced by name:

`value1`, `helloworld`, `hello_world` are all variable names. Note that
uppercase and lowercase letters, numbers, and the underscore are the only
acceptable symbols. Unlike some other languages, the dollar sign is not a legal
character.

### Children And Elements

**TO BE IMPLEMENTED LATER**
Variables can have children, which can be referred to through the dot operator:
`value1.child.grandchild`, `helloworld.world.continents`

Elements of array variables can be referred to through standard bracket
notation: `value1[0]`, `value2[7]`. Any expression can be used inside the
brackets, as long as it resolves to an int or a float. 
`value[helloworld] == value[4]` Floats will be rounded down. E.G. 
`value[1.5] == value[1]`

## Variables

### Definition

A variable can be defined much like other languages:

`Int number;`

`Int number = 4;`

`Int one = 1, two = 2;`

**TO BE IMPLEMENTED LATER**
Since expressions can result in types, expressions can be used to define the
type of a variable. To do so, surround the expression in curly brackets.

`{two._type} three = 3;`

### Data Types

There are not many builtin data types in Waveguide. The most common ones are
`Bool`, `Int`, and `Float`. They do what they do in other languages. Note that
unlike other languages, they are capitalized. This is in an effort to make the
language more uniform. All data types are capitalized.

**TO BE IMPLEMENTED LATER**
There are several builtin datatypes that are only available at compile time:
`_Function`, `_DataType`, `_Lambda`. Because they are only available at compile
time, they are prefixed with an underscore. Whenever a variable with one of
these types is referenced in the code, its value must be determinable at compile
time. This means that the following code is valid:
```rust
fn main {
    _DataType type;
    if(true) {
        type = Float;
    } else {
        type = Int;
    };
    {type} variable = 1;
    print(variable);
}
```
Since the inputs to `if` are able to be determined at compile time, its effect
can be determined at compile time, in turn allowing the value of `type` to be 
determined, making the type of `variable` known at compile time. This is the
biggest strength of waveguide, allowing for features that would normally require
runtime type information without the overhead of RTTI. However, because RTTI is
not used, the following is not valid:
```rust
fn main {
    _DataType type;
    if(randomBool()) {
        type = Float;
    } else {
        type = Int;
    };
    {type} variable = 1;
    print(variable);
}
```
This will cause an error because the inputs to the `if` call cannot be
determined at compile time, yet its lambdas are manipulating values that are
only available at compile time.

### Array Data Types
Array types are defined with a syntax that may seem backwards compared to other
languages:
```rust
[5][4][3]Int int_array_3d;
```
There is a good reason for this. First, an explanation of exactly what this
example is describing: a variable that holds a 5-element array with elements of
type (4-element array of type (3-element array of type Int)). From this, we can
see that the 5-element array is the biggest type, and Int is the smallest type.
If we sort these vertically by size, we get the following diagram:
```rust
[5]
   [4]
      [3]
         Int
```
Nice and ordered. If we were to do it like other languages:
```rust
Int[5][4][3] bad_array;
```
We get this diagram:
```rust
   [5]
      [4]
         [3]
Int
```
Well, that's not too terrible. A little unintuitive, but not enough to warrant
completely reversing the syntax. However, if we look at template parameters,
things start to get weird. Let's suppose that for this example,
```rust
T === [3]Int;
```
In other words, the type `T` represents a 3-element array of `Int`s. If we
wanted to create our original data type (`[5][4][3]Int`) using `T` and our
better syntax, it would look like this:
```rust
[5][4]T;
```
It is easy to determine the actual data type this resolves to just by swapping
out T with what it represents:
```rust
[5][4][3]Int;
```
This again gives us the nice ordered diagram from the beginning. Now let's try
to do the same using a more traditional syntax:
```rust
T === Int[3];
T[5][4];
```
This fundamentally represents the same data type. `T` is a 3-element array of
`Int`s. The final data type is a 5-element array of 4-element arrays of type T.
However, if we try the simple trick of replacing the template parameter with
what it represents to determine its actual data type, we get:
```rust
Int[3][5][4];
```
Yuck. And to quantify that yuckiness:
```rust
      [5]
         [4]
   [3]
Int
```
It's all over the place. By using a traditional array syntax, it opens up
the possibility of specifying array sizes in arbitrary order, which is very 
unintuitive. This is why the backwards-looking syntax was selected.

One final note on arrays is that there are no dynamically-sized arrays. All
arrays must have a size defined at compile time. Because of waveguide's builtin
compile-time simplification, any expression that can be resolved at compile time
can be used to specify the size of an array. This can be as simple as:
```rust
Int[4] array;
```
As idiomatic as:
```rust
const FILTER_SIZE = 512;
Int[FILTER_SIZE] kernel;
```
or as complex as:
```rust
fn fibbonacci(Int iterations):(Int output) {
    Int before_output = 1, temp;
    output = 1;
    repeat(iterations) {
        temp = output;
        output += before_output;
        before_output = temp;
    };
}
Int[fibbonacci(12)] fibbonacci_array;
```
Note that, unlike other languages, there is no special syntax needed to make
the function `fibbonacci` work at compile time. That's the power of waveguide's
built-in interpreter.

## Expressions

### Math

Pretty simple, like most languages:

`a + b` is addition

`a - b` is subtraction

`a * b` is multiplication

`a % b` is modulo (remainder), works for both floats and ints.

`a ** b` is power (a to the power of b.)

Slight deviation from most languages, more pythonic:

`a / b` is floating-point division, the operands will be cast to float.

`a // b` is integer division, the operands will be cast to int.

### Values

Any value is also an expression.

### Comparison

Like most languages again:

`a == b` checks if a is equal to b

`a != b` checks if a is not equal to b

`a > b` checks if a is greater than b

`a < b` checks if a is less than b

`a >= b` checks if a is greater than or equal to b

`a <= b` checks if a is less than or equal to b

### Logic

More pythonic with this one, to reserve more symbols for mathy stuff:

`a and b` performs a logical short-circuit and operation.

`a or b` performs a logical short-circuit or operation.

`a xor b` performs a logical short-circuit xor operation.

`a nand b` performs a logical short-circuit nand operation.

`a nor b` performs a logical short-circuit nor operation.

`a xnor b` performs a logical short-circuit xnor operation.

The bitwise variants just add 'b' on to the front of the operation name, for 
example: `a band b` does a bitwise and of a and b. Another example is 
`a bxnor b` which, if you ever use, I want to see what bizarre set of 
circumstances lead to something like that.

### Arrays
Operations are performed elementwise on arrays. For example:
```rust
[1, 2, 3] + [4, 4, 4] == [5, 6, 7];
[1, 0, 1, 0] * [1, 2, 3, 4] == [1, 0, 3, 0];
```
As will be explained in the casting section, the following is also true:
```rust
[1, 2, 3] * 4 == [4, 8, 12];
```

## Casting
Whenever a variable needs to be a different type for an operation, casting 
occurs. You may be familiar with this from its basic form in other languages.
For example, when executing the expression `14 + 5.3`, `14` is first cast to
a float so it can be added to the other float. This is because the operands are
of type `Int` and `Float` respecively. Of those two, `Float` is the most precise
so the other variable must be cast to `Float` before the addition can occur.
Casting can also occur when a specific type is required for a function. For 
example, consider:
```rust
fn factorial(Int iterations):(Int output) {
    output = 1;
    repeat(iterations) (factor) {
        output = output * factor;
    };
}
```
If you were to input a `Float` into this function:
```rust
factorial(12.3);
```
It must first be cast to an `Int` before the function can run.

### Biggest Common Type Rules
When two values must have the same type (such as with arithmetic operations),
these rules are used to determine what type they should each be cast to so that
the operation can occur. It is often the case that this common type will be the
same as the type of one of the inputs, though there are many cases where the
common type is not shared by any of the inputs, requiring all inputs to be cast
to it. `T` and `U` will be used to represent any type, and `x`, `y`, and `z` 
will be used to represent arbitrary numbers. The format for these rules is
`{type1} + {type2} -> {common type}`. If the common type specifies something
like `T + U`, it means to recursively apply the rules on `T` and `U` to
determine the common type. All rules are commutative, meaning the oprands can be
applied in any order.

1. `T + T -> T`
2. `Float + Int -> Float`
3. `[x]T + U -> [x]{T + U}`
4. `[x]T + [x]U -> [x]{T + U}`

These rules are applied in order and recursively. If the end of the list is
reached because none of the rules apply, the cast is considered invalid and a
compile-time error will be thrown. It is then up to the programmer to manipulate
the inputs such that they match the rules.

### Biggest Common Type Examples:

Consider `Int + Float`.
- According to rule 2, this becomes `Float`.

Consider `[5]Int + Float`.
- According to rule 3, this becomes `[5]{Int + Float}`.
- According to rule 2, this becomes `[5]Float`.
- Note that this type is different from both inputs.

Consider `[5]Int + [5]Int`.
- According to rule 1, this becomes `[5]Int`.

Consider `[10][3]Int + Float`.
- According to rule 3, this becomes `[10]{[3]Int + Float}`.
- According to rule 3, this becomes `[10][3]{Int + Float}`.
- According to rule 2, this becomes `[10][3]Float`.

Consider `[10][3]Int + [3]Int`.
- According to rule 3, this becomes `[10]{[3]Int + [3]Int}`.
- According to rule 1, this becomes `[10][3]Int`.

Consider `[5]Int + [3]Int`.
- There is no rule that matches this situation, so the cast is invalid. The
programmer may extend the second array to have 5 elements, or they may take the
average of both arrays to get two floating point values, or any other number of
operations to make their data types compatible.

### Casting Rules
These are rules governing what can be cast to what, and how that casting occurs.
`T` and `U` will be used to represent any type, and `x`, `y`, and `z` will be 
used to represent arbitrary numbers. These rules are *not* commutative.

1. `T -> T`: No operation is performed.

2. `Int -> Float`: Simple conversion, the decimal part is set to 0.

3. `Float -> Int`: The decimal part is discarded.

4. `[x]T -> [x]U`: Every element of the array is cast using the rule for
`T -> U`.

5. `T -> [x]U`: The value of the variable is cast using the rule for `T -> U`, 
and then copied `x` times to fill the array.

6. `[1]T -> U`: The first value of the array is cast using the rule for `T -> U`
and then copied to the output.

These rules are applied in order and recursively to any situation requiring
casting. If the end of the list is reached and none of the rules apply, the cast
is illegal and will produce a compile-time error. In this case, the programmer
must manually convert the data either to the target type or to a type that
can be automatically casted to the target type.

### Casting Examples
These are examples of syntactically valid expressions and how they are 
interpreted through the combination of the biggest common type rules and the
casting rules.

Consider `1 + 2`. 
- Looking at the types, the common type is `Int + Int`.
- According to BCT rule 1, this becomes `Int`.
- The cast of the first operand is then `Int -> Int`.
- According to casting rule 1, nothing happens.
- The cast of the second operand is `Int -> Int`.
- Again, according to casting rule 1, nothing happens.
- The final expression internally looks like `1 + 2`.

Consider `1 + 2.0`
- The common type is `Int + Float`.
- According to BCT rule 2, this becomes `Float`.
- The cast of the first operand is then `Int -> Float`.
- According to casting rule 2, it is given an empty decimal section, making 
`1.0`.
- The cast of the second operand is `Float -> Float`.
- According to casting rule 1, nothing happens.
- The final expression internally looks like `1.0 + 2.0`.

Consider `1 + [5, 6]`
- The common type is `Int + [2]Int`.
- According to BCT rule 3, this becomes `[2]{Int + Int}`.
- According to BCT rule 1, this becomes `[2]Int`.
- The cast of the first operand is then `Int -> [2]Int`.
- According to casting rule 5, the single integer is copied to fill a 2-element
array, making `[1, 1]`.
- The cast of the second operand is `[2]Int -> [2]Int`.
- According to casting rule 1, nothing happens.
- The final expression internally looks like `[1, 1] + [5, 6]`.
- The result of this expression is `[6, 7]`.

Consider `[[2, 1], [4, 3]] * 1.5`
- The common type is `[2][2]Int + Float`.
- According to BCT rule 3, this becomes `[2]{[2]Int + Float}`.
- According to BCT rule 3, this becomes `[2][2]{Int + Float}`.
- According to BCT rule 2, this becomes `[2][2]Float`.
- The cast of the first operand is `[2][2]Int -> [2][2]Float`
- According to casting rule 4, the cast `[2]Int -> [2]Float` should be applied
to each element.
- According to casting rule 4, the cast `Int -> Float` should be applied to each
sub-element.
- Overall, this makes the first operand `[[2.0, 1.0], [4.0, 3.0]]`.
- The cast of the second operand is `Float -> [2][2]Float`.
- According to casting rule 5, the cast `Float -> [2]Float` should be applied
and the resulting value be copied twice to fill the array.
- According to casting rule 5, that cast is fulfilled by first applying the cast
`Float -> Float` (in which nothing happens), and then the resulting value
copied twice to fill the array.
- Overall, this makes the second operand `[[1.5, 1.5], [1.5, 1.5]]`.
- The result of this expression is then `[[3.0, 1.5], [6.0, 4.5]]`.

Consider `[1, 2, 3] + [4, 5]`
- The common type is `[3]Int + [2]Int`.
- There is no BCT rule that can be applied to this, so the cast is invalid.

## Functions

Functions are the weirdest thing about waveguide. For loops are functions. If
statements are functions. Regular functions are functions, too. So let's look
at examples:

### Declaration

(Ignore the fact that everything is colored for the 'rust' language)

```rust
fn double(Int input):(Int output) {
    output = input * 2;
    return();
}
```
Functions are declared similarly to rust, by prefixing the definition with the
`fn` keyword. The keyword is followed by the name of the function, then a
description of the inputs and outputs of the function. After that, a code block
surrounded in curly brackets contains the actual code for the function. Note
that to return a value, there is no 'return' keyword. Instead, we use the
special method 'return'. This only looks like a function though, as it has no
programatic definition. It only exists in the compiler. 

```rust
fn double(Int input):(Int output) {
    return(input * 2);
}
```
It is often the case that we can find the value of the outputs at the same time
that we want to return. In this case, the return function can be used similarly
to other languages. It will automatically set the values of all output values
using the input argments it receives.

```rust
fn doubleAndTriple(Int input):(Int doubled, Int tripled) {
    return(input * 2, input * 3);
}
```
This works for multiple outputs, too.

```rust
fn add(Int a, Int b):Int {
    return(a + b);
}
```
There are many times where we do not care about the name of the output. Though
it is usually recommended to provide a name for readability reasons, there are
some methods that are so self-explanatory that they do not require one. In this
case, the type of the output can be provided without parenthesis. This will
internally generate a variable with a syntactically invalid name, so the only
way to set it is with the return function. This syntax is most similar to the
single-return-only paradigm of many popular languages.

### Usage

`result = sin(1.0);` Pretty typical syntax here, computes the sine of 1.0.

`sin(1.0):(result);` Does the same thing as before, just with different syntax.

`sort(3.0, 1.0):(biggest, smallest);` This will call the method `sort`, giving
it the inputs `3.0` and `1.0`, putting the outputs of the function call in the
variables `biggest` and `smallest`. This is one of the really useful things
about functions in waveguide, there is minimal overhead to add multiple outputs
to a function.

`sin(1.0):(exampleArray[5]);` Anything you can put on the left of an equals
sign, you can put into the output of a function call.

`sin(1.0):(Float sineOutput);` This includes variable declarations. The scope
of the variable will be the same as if it was declared on a line above the
function call and then only the variable name was in the output section of the
function call.

`if(true) { stuff(); };` `if` is a function. true is provided for the first
argument. The section of code after it is a **lambda**, which is like a
miniature function. It can contain any code that a function body can, except 
that if you want to 'return' from a lambda, you use `break()` instead of
`return()`. If you were to use `return()`, it would use the return method from
whatever function you are in. For example, if you put `return()` in an `if`
call inside the definition for `main`, then it would cause the `main` function
to return. `break()` would return from the lambda inside the `if` function.
Note that, unlike other languages, there *must* be a semicolon at the end of the
`if` call, since it is a function in waveguide, while in other languages it is a
statement.

`repeat(10) (Int iteration) { print(iteration); };` Lambdas can have inputs and
outputs. They are specified just like function inputs and outputs.

`repeat(10) (iter) { print(iter); };` A function author can specify what types
are required for inputs or outpus of lambdas, so the type can be ommitted for 
brevity in most cases.

`if(false) { stuff(); } else { things(); };` This is a bigger example of the
`if` function. In this case, `else` is what's known as an 'adjective'.
Adjectives are specified by the author of the function, and are used to modify
either the behavior of the overall function or the behavior of lambdas coming
after the adjective. In this case, the `else` adjective signals to the `if`
function that the lambda containing the call to `things()` should only be
executed if all the other conditions are false.

`if(false) { stuff(); } elif(true) { things(); } else { nothing(); };` This is
a complete example of the `if` function. `elif` is another adjective that
signals that the code block containing the call to `things()` should only be
executed if the condition (`true`) is true and all the conditions before it are
false. The code block with the `else` adjective would, in this case, not run, 
because the condition for the `elif` adjective is `true`, so not all the
conditions before it are `false`.

`try { stuff(); };` is also a valid function call. In this case, there are no
arguments. This would be equivalent to `try() { stuff(); };`.

Note that although the above suggests you could do something like 
`really long function call thing with no arguments or code blocks;`, (where 
`really` is the name of the function, and the remainder of the words are 
adjectives) this would cause ambiguity in the grammar, making the compiler 
impossible to write. Instead, a restriction is enforced that every function call
must either specify inputs, outputs, or a code block with no adjectives before 
it. This covers 99.9% of use cases. These are all examples of legal calls: 
`func {} adj1 adj2;`, `func() adj1 adj2;`, `func:() adj1 adj2;`. These are not 
legal: `func adj1;`, `func adj1(in1, in2) { };`, `func;`. This illustrates the
ambiguity problem, because `func adj1;` actually creates a variable named `adj1`
of type `func`, and `func;` is a valid statement which has no effect (and will
produce a compiler warning.) This is because all expressions followed by
semicolons are valid statements, due to the fact that many have side effects. 
(Remember, `if` is technically just a function call, making it an expression.)

## Templates

### Introduction
Waveguide has a powerful template syntax that allows for a large amount of 
flexibility when writing functions. First, let's consider a function that adds
two values of arbitrary types:
```rust
fn add(T? value_one, T? value_two):T {
    return(value_one + value_two);
}
```
The question mark after the letter T indicates that it is a template parameter.
Let's look at what happens when this function is called:
```rust
Int result = add(12.3, 3);
```
First the actual data type represented by T must be determined. BCT rules are
used to determine this. The only types considered in the BCT calculation are
the types of parameters marked with the question mark. In this case, only the
type of the two inputs (`Float` and `Int` respectively) are considered.
According to BCT rules, `T` resolves to `Float`. The output type must then be
`Float`. 

### Using Template Types In The Body
Using a question mark internally declares a new type, which you can use like any
other type name. (This is why `T` can be used without a question mark as the
output of our example function.) For example, consider this overly verbose
addition function:
```rust
fn overly_verbose_addition(T? input1, T? input2):T {
    T result = input1 + input2;
    return(result);
}
```
Note that curly brackets did not have to be used since `T` is a fully-fledged 
type name, not just a variable holding a type. Now consider this useless 
addition of an array:
```rust
fn overly_complicated_addition(T? input1, T? input2):T {
    [3]T buffer;
    buffer[0] = input1;
    buffer[1] = input2;
    buffer[2] = buffer[0] + buffer[1];
    return(buffer[2]);
}
```
Also, since `T` is a type name, you can declare an array of `T`. Now consider
we call our overly-complicated addition function like this:
```rust
[256]Int buffer1, buffer2, output;
overly_complicated_addition(buffer1, buffer2):(output);
```
In this case, `T` resolves to `[256]Int`, making our function body equivalent
to this:
```rust
fn overly_complicated_addition([256]Int input1, [256]Int input2):[256]Int {
    [3][256]Int buffer;
    buffer[0] = input1;
    buffer[1] = input2;
    buffer[2] = buffer[0] + buffer[1];
    return(buffer[2]);
}
```
As you can see, the function makes sense just by dropping `[256]Int` in place of
`T`, demonstrating the advantages of the backwards array declaration syntax.

### Array Templates
**TO BE IMPLEMENTED LATER.**
Not to be confused with the previous example, array templates are templates that
look specifically for arrays of types. There are a handful of template features
we can use to specify arrays as inputs:
```rust
fn accepts_triplet([3]T? input) { ... }
```
This accepts a 3-element array of an unknown type. This function could be called
like so:
```rust
accepts_triplet([1, 2, 3]); # T == Int
accepts_triplet([1., 2., 3.]); # T == Float
accepts_triplet([[1], [2], [3]]); # T == [1]Int
```
The size of the array can be specified using any expression that can be resolved
at compile time. The size can also be a template parameter itself:
```rust
fn accepts_array([SIZE?]T? input) { ... }
accepts_array([1, 2, 3]); # T == Int, SIZE == 3
accepts_array([[1, 2, 3]]); # T == [3]Int, SIZE == 1
```
You can even combine this with any expression that can be resolved at compile
time:
```rust
fn accepts_array([SIZE?]T? input1, [fibbonacci(SIZE)]T? input2) { ... }
accepts_array([1, 2], [3]); # T == Int, SIZE == 2
accepts_array([[1, 2, 3]], [[3]]); # T == [3]Int, SIZE == 1
```
However, make note that only the first appearence of `SIZE` has a question mark.
This will be explained soon.

Unlike type template parameters, size template parameters do no kind of casting 
calculation to determine their final value. Instead, they must be matched 
exactly by the input. This is due to the ambiguity and resulting illegality of 
casting between two arrays of different sizes. For example:
```rust
fn compare_arrays([SIZE?]T? input1, [SIZE?]T? input2);

compare_arrays([1, 2], [1, 3]); # Valid.
compare_arrays([1, 2], [3.0, 4.0]); # Valid (T becomes Float, input1 gets cast.)
compare_arrays([1], [2, 3]); # INVALID! The possible values for SIZE are 1 and 2
```
Let's look at what happens when our template accepts arrays of different depths:
```rust
fn find_element([SIZE?]T? array, T? element):Int {
    repeat(SIZE) (index) {
        if(array[index] == element) {
            return(index);
        };
    };
}

[10]Int array;
Int element;
find_element(array, element); # SIZE == 10, T == Int

[5][2]Int array;
[2]Int element;
find_element(array, element); # SIZE == 5, T == [2]Int

[5][4]Int array;
[2]Int element;
find_element(array, element); # Illegal, inputs for T are [4]Int and [2]Int,
                              # which does not match any BCT rule.

[5][1]Int array;
[4]Int element;
find_element(array, element); # SIZE == 5, T == [4]Int
                              # (array is cast to [5][4]Int)

[5][4]Int array;
Int element; # or [1]Int element, it will work the same way.
find_element(array, element); # SIZE == 5, T == [4]Int
                              # (element is cast to [4]Int)
```
One final piece of syntax allows accepting arrays of variable depth:
```rust
fn arbitrary_depth([SIZE?[DEPTH?]]T? input) { ... }

[4][2]Int buffer;
arbitrary_depth(buffer); # SIZE == [4, 2], DEPTH == 2, T == Int
[4][4][4]Int cube;
arbitrary_depth(cube); # SIZE == [4, 4, 4], DEPTH == 3, T == Int
Int value;
arbitrary_depth(value); # SIZE == [], DEPTH = 0, T == Int
```
`SIZE` is now an array and `DEPTH` is an `Int`. You can use these both in other
parts of the function signature:
```rust
fn arbitrary_depth([SIZE?[DEPTH?]]T? input):([DEPTH]T output) { ... }
```
One final note: just like type template parameters, size template parameters can 
be used in the function body:
```rust
fn sum([SIZE?]T? input):(T output) {
    output = 0;
    repeat(SIZE) (index) {
        output = output + input[index];
    };
}
```
In this case, suppose our input is of type `[10][3]Int`. The code above would be
equivalent to:
```rust
fn sum([10][3]Int input):([3]Int output) {
    output = 0; # This will be automatically casted to [0, 0, 0].
    repeat(10) (index) {
        output = output + input[index];
    };
}
```

### Where To Use The Question Mark
The question mark is only used for parts of a template expression that should be
used to determine the value of a particular template parameter. Outputs of a 
function cannot have question marks, and most inputs usually use question marks.
Question marks in templates for inputs are only invalid when the template
parameter is being used as an input for an expression. For example:
```rust
fn illegal_question_mark([factorial(SIZE?)]Int input) { ... }
```
This is because the question mark indicates that the compiler should try and
determine the value of that parameter based on the actual data type that is
inputted. The above syntax says that the compiler must find a value for `SIZE`
such that `factorial(SIZE)` equals the size of the array that was given to the
function. This requires the compiler to perform the reverse of the `factorial`
function. Since this problem is not solvable in the general case in a reasonable
amount of time, it has been chosen to not implement any kind of solver into the
compiler for these situations. Instead, the question mark should only be used in
cases where the compiler can trivially determine its value:
```rust
fn legal_question_mark([SIZE?]Int input) { ... }
fn legal_question_mark_2([SIZE?]Int input, [factorial(SIZE)]Int input2) { ... }
```
Note that this restriction does not forbid complex template parameters. In the
case of `legal_question_mark_2`, the compiler can still enforce the oddball 
factorial size constraint because it can already trivially determine the value
of `SIZE` from the first input. Complex constraints are only a problem when
every occurence of a template parameter involves some complicated expression.
This case can, however, always be avoided by algebraic manipulation by the
programmer. For example, consider:
```rust
fn illegal_signature([SIZE? + 1]Int input1, [SIZE? - 1]Int input2) { ... }
```
Both inputs are invalid because they are using the question mark inside an 
expression. The above code can be changed to something like this:
```rust
fn legal_signature([SIZE?]Int input1, [SIZE - 2]Int input2) {
    const REAL_SIZE = SIZE - 1;
}
```
Note that the question mark no longer appears in any expressions, and the size
constraints behave identically to the illegal ones in `illegal_signature`.
Additionally, the constant `REAL_SIZE` will have the same value as `SIZE` would
have for the `illegal_signature` version.

These restrictions also apply to template type parameters, although since types
are rarely used in expressions, it is more common to see every instance of a
type parameter suffixed with a question mark:
```rust
fn example([SIZE?]T? array1, [SIZE + 5]T? array2) { ... }
```
Also note that question marks can never be used in definitions of outputs. All
outputs must be deterministic to help alleviate ambiguity:
```rust
fn legal([SIZE?]T? input):[SIZE]T { ... }
fn illegal([SIZE?]T? input):[OUT_SIZE?]U? { ... }
```
