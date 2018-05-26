import lexical_analyzer

program = '''
sort(Int a,Int b):(Int c, Int d) {
    if(a > b) {
        c = a;
        d = b;
    } else {
        c = b;
        d = a;
    }
}
Int a = 5, b = 2;
Int d = sort(a, b):(Int c, return);
typedef {Float[256][2] left, right=2} Buffer;
Buffer buf = {left=0, right=0};
Int[12] array = [1, 2, 3, 4, a, b, c, d, (1 + (2))], array2 = 0;
Float e = cos(12.0), f = sin(array[8] + array[7]), g = pow(4.0, 5.0 + 6.0 * 0.7);
add(Int a, Int b) {
    return a + b;
}
sub(Int a, Int b):Int {
    return a - b;
}
Int n = add(sub(add(1, 2), add(3, 4)), sub(5, 2));
Int o = 1 + 2 * (3 + 4 * (5 + 6 * (7 + 8)))
'''

print('[[INPUT]]')
print(program)

def compile(code):
    passes = [lexical_analyzer.convert_to_tokens]
    for p in passes:
        code = p(code)
    return code

print('[[OUTPUT]]')
print(compile(program))