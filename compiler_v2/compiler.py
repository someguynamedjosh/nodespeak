import lexical_analyzer
import syntax_analyzer
from time import time

program = '''
def sort(Int a,Int b):(Int c, Int d) {
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
#typedef {Float[256][2] left, right=2} Buffer;
#Buffer buf = {left=0, right=0};
Int[12] array = [1, 2, 3, 4, a, b, c, d, (1 + (2))], array2 = 0;
Float e = cos(12.0), f = sin(array[8] + 1), g = pow(4.0, 5.0 + 6.0 * 0.7);
def add(Int a, Int b):(Int return) {
    return a + b;
}
def sub(Int a, Int b):Int {
    return a - b;
}
Int n = add(sub(add(1, 2), add(3, 4)), sub(5, 2));
Int o = 1 + 2 * (3 * 4 + (5 + 6 * (7 + 8)));
'''

print('[[INPUT]]')
print(program)

def compile(code):
    passes = [lexical_analyzer.convert_to_tokens, syntax_analyzer.build_ast]
    for p in passes:
        code = p(code)
    return code

print('[[OUTPUT]]')
print(compile(program))

start_time = time()
end_time = start_time + 0.5
iterations = 0
while time() < end_time:
    for i in range(10):
        compile(program)
    iterations += 10
end_time = time() # Get the actual time we stopped.
ms = (end_time - start_time) / iterations * 1000 # Milliseconds per compile.
print('Compiling the program takes', int(ms * 10) / 10, 'ms.')
print('(Completed', iterations, 'in', (end_time - start_time), 'seconds.')