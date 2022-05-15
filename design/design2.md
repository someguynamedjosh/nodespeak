```py
my_func = node {
    # input data: Any;
    input data;

    # If an output has a type annotation, it only serves as a check to see that
    # the output is what we expect (output types can be automatically computed
    # from input types.)
    output data2;

    data2 = 2 * data;
}

# Using it in another func that returns [2, 4, 6]:
other_func = node {
    output data;

    data = my_func([1, 2, 3]);
}

sum_prod = func {
    input a;
    input b;

    outupt sum;
    output prod;

    sum = a + b;
    prod = a * b;
}

# Using it in another func that returns 6.
returns_six = func {
    output six;

    local three, local two = sum_prod(1, 2);

    six = three * two
}

# Compile error, LENGTH is not a valid value for Array::LENGTH.
my_func = node {
    ct_input LENGTH: Int;
    input data: Array(Any, LENGTH);
}

my_func = node {
    ct_input LENGTH: InRange(1, 0xFFFF_FFFF);
    # Or: ct_input LENGTH: Size;
    input data: Array(Any, LENGTH);
}

# Returning compile-time-only values from a func:
example = func {
    ct_input QUANTITY: Int;
    input amount: Int;

    output Buffer = Array(InSet(Int, Float), QUANTITY);

    output add = func {
        input original: Int;
        output sum = original + amount;
    }
}

# Using it...
usage = func {
    input another_amount: Int;
    local Buffer, local add = example(12, another_amount);
    input buf: Buffer;

    output result = add(buf);
}

# Doing arithmetic on types produces the type that would result from such an
# operation.
Int + Float == Float;
Array(Int, 1) + Array(Float, 4) == Array(Float, 4);
Array(Int, 2) + Array(Int, 3) == Never;
Int > Int == Bool;
InSet(5) > InRange(1, 4) == InSet(TRUE);
```