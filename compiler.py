import lexical_analyzer
import tokenifier
import desugarifier
import scopifier

from time import time

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
typedef {Float[256][2] left, right} Buffer;
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

#program = 'typedef {Int a = 1, b = 2; Float c;}[3][4] monstrosity;'

print('[[INPUT]]')
print(program)

def compile(code):
	passes = [lexical_analyzer, tokenifier, desugarifier, scopifier]
	for p in passes:
		code = p.process(code)
	return code

print('[[OUTPUT]]')
print(compile(program))


print('Starting timed test...')
start = time()
iters = 0
TIME_LIMIT = 1.0 # Don't test for longer than a second.
while time() - start < TIME_LIMIT:
	compile(program)
	iters += 1
elapsed = time() - start
print('[[RESULT]]')
print(int(elapsed / iters * 1000000) / 1000, 'ms per compile')
