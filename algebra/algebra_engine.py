from functools import reduce
import math
import re

class TokenQuery:
    def __init__(self, pattern, sub_queries=[], order_matters=False, quantity_matters=False):
        self.pattern = pattern
        self.sub_queries = sub_queries
        self.order_matters = order_matters
        self.quantity_matters = quantity_matters
        
    def check_args(self):
        return (len(self.sub_queries) > 0) or (self.quantity_matters)
    
    def __repr__(self):
        tr = self.pattern
        if(self.sub_queries):
            tr += '{' + ', '.join([str(i) for i in self.sub_queries]) + '}'
        return tr

TQ = TokenQuery

class Token:
    def __init__(self, contents):
        self.contents = contents
    
    def __name__(self):
        return self.contents
    
    def matches(self, pattern):
        # print(self, '(' + self.__name__() + ')', '=?=', pattern)
        if(not re.match(pattern.pattern + 'ENDENDEND', self.__name__() + 'ENDENDEND')):
            return False
        if(pattern.check_args()):
            if(type(self.contents) is not list):
                return False
            if((pattern.quantity_matters) and (len(pattern.sub_queries) != len(self.contents))):
                return True
            if(pattern.order_matters):
                for i, sq in enumerate(pattern.sub_queries):
                    if(not self.contents[i].matches(sq)):
                        return False
                return True
            else:
                used = []
                for sq in pattern.sub_queries:
                    for i, arg in enumerate(self.contents):
                        if(arg.matches(sq) and (i not in used)):
                            used.append(i)
                            break
                    else:
                        return False
                return True
        else:
            return True
        print('Unknown error when matching against token query.')
        return False
    
    def __repr__(self):
        return self.contents

class Operator(Token):
    pass

class UnaryOperator(Token):
    pass

class Phrase(Token):
    pass

class Number(Token):
    def __repr__(self):
        return str(self.contents)
    
    def __name__(self):
        return 'num_' + str(self.contents)

class Variable(Token):
    def __name__(self):
        return 'var_' + self.contents

class Group(Token):
    def __init__(self, contents):
        self.contents = contents
        
    def __repr__(self):
        return '(' + ', '.join([str(t) for t in self.contents]) + ')'
    
    def __name__(self):
        return 'group'
    
class Function(Group):
    def __init__(self, name, contents):
        self.name, self.contents = name, contents
        
    def __repr__(self):
        return self.name + super().__repr__()
    
    def __name__(self):
        return 'func_' + self.name

def tokenize(formula, operator_list):
    formula = [Phrase(formula)]
    operator_list.sort(key=lambda e: -len(e))
    for operator in operator_list:
        oldFormula = list(formula)
        formula = []
        for chunk in oldFormula:
            if(type(chunk) is Phrase):
                bits = chunk.contents.split(operator)
                for i, bit in enumerate(bits):
                    if(bit):
                        formula.append(Phrase(bit))
                    if(i != len(bits) - 1):
                        formula.append(Operator(operator))
            else:
                formula.append(chunk)
    return formula

def group(tokens, name = None):
    layer = 0
    start = 0
    nname = None
    output = []
    for i in range(len(tokens)):
        if(tokens[i].contents == '('):
            layer += 1
            if(layer == 1):
                start = i
                if((i > 0) and (type(tokens[i-1]) is Phrase)):
                    nname = tokens[i-1].contents
                    output = output[:-1]
                else:
                    nname = None
        elif(tokens[i].contents == ')'):
            layer -= 1
            if(layer == 0):
                output.append(group(tokens[start+1:i], nname)) # Skip parenthesis
        elif(layer == 0):
            output.append(tokens[i])
    if(name):
        return Function(name, output)
    else:
        return Group(output)

def wrap_func(fname, token):
    if(type(fname) is Group):
        return Function(fname, token.contents)
    else:
        return Function(fname, token)

def pre_funcify(tokens):
    output = []
    for i in range(len(tokens)):
        token = tokens[i]
        # / normally means divide. We want it to mean reciprocal, because that is easier to parse.
        if(token.contents == '/'):
            output += [Operator('*'), UnaryOperator('/')]
        # - can mean subtract or negate. We only want it to mean negate.
        elif(token.contents == '-'):
            if((i > 0) and (type(tokens[i-1]) is not Operator)): # Subtraction
                output += [Operator('+'), UnaryOperator('-')]
            else: # Already negation, no processing necessary.
                output.append(UnaryOperator('-'))
        elif(type(token) is Group):
            output.append(Group(pre_funcify(token.contents)))
        elif(type(token) is Function):
            output.append(Function(token.name, pre_funcify(token.contents)))
        else:
            output.append(token)
    return output

def funcify(tokens):
    output = []
    i = 0
    # Unary operator funcify
    while i < len(tokens):
        token = tokens[i]
        if(type(token) is UnaryOperator):
            if(token.contents == '-'):
                output.append(Function('mul', [Phrase('-1'), tokens[i+1]]))
            elif(token.contents == '/'):
                output.append(Function('recip', [tokens[i+1]]))
            i += 1
        elif(type(token) is Group):
            output += funcify(token.contents)
        elif(type(token) is Function):
            output.append(Function(token.name, funcify(token.contents)))
        else:
            output.append(token)
        i += 1
        
    # Binary operator funcify
    ops = {
        '^': 'pow',
        '*': 'mul',
        '+': 'add'
    }
    ooo = ['^', '*', '+'] # dicts are unordered, order of operations is important.
    for op in ooo:
        while True:
            for i, token in enumerate(output):
                if(token.contents == op):
                    if((type(output[i-1]) is Function) and (ops[op] in ['add', 'mul']) and (output[i-1].name == ops[op])):
                        # Combine multiple terms added or multiplied together.
                        output[i-1].contents.append(output[i+1])
                        output = output[:i] + output[i+2:]
                    else:
                        output = output[:i-1] + [Function(ops[op], [output[i-1], output[i+1]])] + output[i+2:]
                    break
            else:
                break
    return output

def num_parse(token):
    for i, t in enumerate(token.contents):
        if(type(t) is Phrase):
            try:
                token.contents[i] = Number(float(t.contents))
            except:
                token.contents[i] = Variable(t.contents)
        elif(type(t) is Function):
            token.contents[i] = num_parse(t)
    return token

def simplify_constants(token):
    def do_func(name, inputs):
        funcs = {
            'add': lambda i: reduce(lambda a, b: a + b, i),
            'mul': lambda i: reduce(lambda a, b: a * b, i),
            'pow': lambda i: pow(i[0], i[1]),
            'recip': lambda i: 1.0 / i[0],
            'sin': lambda i: math.sin(i[0]),
            'cos': lambda i: math.cos(i[0]),
            'tan': lambda i: math.tan(i[0]),
            'asin': lambda i: math.asin(i[0]),
            'acos': lambda i: math.acos(i[0]),
            'atan': lambda i: math.atan(i[0])
        }
        return funcs[name](inputs)
    
    for i, t in enumerate(token.contents):
        if(type(t) is Function):
            token.contents[i] = simplify_constants(t)
    constants = [t.contents for t in token.contents if type(t) is Number]
    if(len(constants) == len(token.contents)): # All inputs are constant.
        return Number(do_func(token.name, constants))
    elif(len(constants) > 1):
        args = [Number(do_func(token.name, constants))]
        args += [t for t in token.contents if type(t) is not Number]
        return Function(token.name, args)
    else:
        return token

def simplify(token):
    if(type(token.contents) is list):
        for i, t in enumerate(token.contents):
            token.contents[i] = simplify(t)
            
    ot = str(token)
    # (a^b)^c = a^(b*c)
    if(token.matches(
        TQ('func_pow', [
            TQ('func_pow')
        ], order_matters = True))):
        base = token.contents[0].contents[0]
        exps = Function('mul', [token.contents[0].contents[1], token.contents[1]])
        token.contents = [base, exps]
    # (a^b) * (a^c) * ... * (a^z) = a^(b+c+...+z)
    if(token.matches(
        TQ('func_mul', [
            TQ('func_pow'),
            TQ('func_pow')
        ])
    )):
        pows = [i for i in token.contents if i.__name__() == 'func_pow']
        other = [i for i in token.contents if i.__name__() != 'func_pow']
        bases = {}
        for pow in pows:
            bases[str(pow.contents[0])] = []
        for i, pow in enumerate(pows):
            bases[str(pow.contents[0])].append(i)
        token = Function('mul', other)
        for base in bases.keys():
            token.contents.append(Function('pow', [
                pows[bases[base][0]].contents[0], 
                Function('add', [pows[i].contents[1] for i in bases[base]])
            ]))
    # A^b*C^d = A^(b+(log(A)(C))d)
    if(token.matches(
        TQ('func_mul', [
            TQ('func_pow', [
                TQ('num_.*'),
                TQ('.*')
            ]),
            TQ('func_pow', [
                TQ('num_.*'),
                TQ('.*')
            ])
        ])
    )):
        pows = [i for i in token.contents if i.__name__() == 'func_pow']
        other = [i for i in token.contents if i.__name__() != 'func_pow']
        minb = min([pow.contents[0].contents for pow in pows]) # Lowest of all the bases.
        # (log base [minb] via change of base formula)
        parts = [(math.log(pow.contents[0].contents) / math.log(minb), pow.contents[1]) for pow in pows]
        pow_token = Function('pow', [
            Number(minb), 
            Function('add', [
                Function('mul', [
                    Number(part[0]),
                    part[1]
                ])
                for part in parts
            ])
        ])
        if(len(other) == 0):
            token = pow_token
        else:
            token = Function('mul', [pow_token] + other)
    # a^B = a * a * ..[B times].. * a
    if(token.matches(
        TQ('func_pow', [
            TQ('var_.*'),
            TQ('num_.*')
        ], order_matters=True)
    )):
        power = token.contents[1].contents
        if(int(power) == power): # It is an integer.
            if((power > 0) and (power < 16)): # Don't expand stuff with large exponents, for sanity's sake.
                token = Function('mul', [Variable(token.contents[0].contents) for i in range(int(power))])
            elif((power < 0) and (power > -16)):
                token = Function('div', [Number(1.0), Function('mul', [Variable(token.contents[0].name) for i in range(power)])])
    # a^0 = 1
    if(token.matches(
        TQ('func_pow', [
            TQ('.*'),
            TQ('num_0.0')
        ], order_matters=True)
    )):
        token = Number(1.0)
    # a^1 = a
    if(token.matches(
        TQ('func_pow', [
            TQ('.*'),
            TQ('num_1.0')
        ], order_matters=True)
    )):
        token = token.contents[0]
    # a + 0 = a
    if(token.matches(TQ('func_add'))):
        parts = [t for t in token.contents if not t.matches(TQ('num_0.0'))]
        if(len(parts) == 0):
            token = Number(0.0)
        elif(len(parts) == 1):
            token = parts[0]
        else:
            token = Function('add', parts)
    # a * 1 = a
    # a * 0 = 0
    if(token.matches(TQ('func_mul'))):
        if(max([t.matches(TQ('num_0.0')) for t in token.contents])):
            token = Number(0.0)
        else:
            parts = [t for t in token.contents if not t.matches(TQ('num_1.0'))]
            if(len(parts) == 0):
                token = Number(1.0)
            elif(len(parts) == 1):
                token = parts[0]
            else:
                token = Function('mul', parts)
            
    if(type(token.contents) is list):
        for i, t in enumerate(token.contents):
            token.contents[i] = simplify(t)
    
    print(ot, '->', token)
    return token
    
def parse_formula(formula):
    operators = ['+', '-', '*', '^', '/', '(', ')']
    o = formula
    print(o)
    o = tokenize(o, operators)
    print(o)
    o = group(o).contents
    print(o)
    o = pre_funcify(o)
    print(o)
    o = funcify(o)[0]
    print(o)
    o = num_parse(o)
    print(o)
    o = simplify_constants(o)
    print(o)
    old = ''
    while str(o) != old:
        old = str(o)
        o = simplify(o)
        o = simplify_constants(o)
    return o

f = '2^a*4^b'

# 2^a*4^b = 2^a*(2*2)^b = 2^a * (2^b)^2 = 2^a*2^2b = 2^(a+2b)
# A^b*(A^2)^c = A^(b+2c)
# A^b*(A^C)^d = A^(b+Cd)

# A^b*(AC)^d # 2 pows, 1 mult
# A^E = AC, log(A)(AC) = E
# A^b*(A^E)^d = A^(b+Ed) = A^(b+(log(A)(AC))d) # 1 pow, 1 mult, 1 add

f = f.replace(' ', '')

print(parse_formula(f))
#simplify(Function('add', [Number(1.0)]))
