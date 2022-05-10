# Expressions

```python
# Assign 5 to x.
(x) 5;
# Assing y the sin of 3.
(y) sin(3);
(y) SIN 3;
# Swap a and b.
(a, b) (b, a);
# A block which returns 3 for a and 5 for b
{
    RETURNING a, b;
    (a) 3;
    (b) 5;
}
# Same thing, but assigning those values to variables.
(will_be_3, will_be_5) {
    RETURNING a, b;
    (a) 3;
    (b) 5;
};
# A function which returns a * b and a + b
(pm) (a, b) => (a * b, a + b);
(pm) (a, b) => {
    RETURNING prod, sum;
    (prod) a * b;
    (sum) a + b;
};
# With annotated types
(pm) (a: Int, b: Int) => {
    RETURNING prod, sum;
    (prod) a * b;
    (sum) a + b;
};
# Using that function.
(will_be_10, will_be_7) pm(2, 5);
(will_be_1) (INLINE, will_be_2) (1, 2);
(will_be_1, will_be_2) (INLINE, INLINE) (1, 2);

# All of these are compile-time-only expressions that can be used as types of
# parameters.
AllInRange(0, 100)
AllInSet(1, 2, 5)
ALL
ANY

# Examples:
# A function accepting an array which must have either 1 or 512 elements. The
# body must work for both cases.
(s: AllInSet(1, 512), a: Array(Int, s)) => ()
# Shortcut for previous.
(a: Array(Int, AllInSet(1, 512))) => ()
# A function which works for zero or more kinds of arrays.
(a: Array(Int, ANY)) => ()
# A function which specifically works for arrays of at least size 4.
(a: Array(Int, ANY)) => a(3)
# A function which works for all kinds of arrays.
(a: Array(Int, ALL)) => ()
# A function which adds values of any two compatible types.
(a: ANY, b: ANY) => a + b
(a, b) => a + b

# A function which accepts functions that return nothing.
(f: ()Function) => f(3)
# A function which accepts functions that return an integer and any compatible
# type.
(f: (Int, ANY)Function) => f(3)
# Examples of using functions that accept functions.
repeat(5, (i) => log(i))

# The most useful statement.
OVER (array1, array2, array3) {
    (array1) array2 + array3;
}
# Well technically you could just do:
(array1) array2 + array3;
# But "over" lets you do weird things like this:
(array1) [0, 0, 0, 0, 0]
(scalar) 0
OVER (array1, scalar) {
    (scalar) scalar + 1
    (array1) scalar
}
# Now array1 = [1, 2, 3, 4, 5]
# Technically putting "scalar" in the parentheses is optional.

# A bit weirder:
(array1) [[0, 0], [0, 0], [0, 0]]
(tracker) [0, 5]
OVER (array1, tracker) {
    (tracker) tracker + 1;
    (array1) tracker;
}
# Now array1 = [[1, 6], [2, 7], [3, 8]].
# Putting "scalar" in the parentheses is not optional if this behavior is desired.

# This is useful for things like:
(array1) [[0, 0], [0, 0], [0, 0]]
(is_left) [TRUE, FALSE]
OVER (array1, is_left) {
    IF (is_left) {
        (array1) one_formula;
    } ELSE {
        (array1) another_formula;
    }
}

# By the way, we have if statements.
IF (thing) expr1 ELSE expr2;
```
