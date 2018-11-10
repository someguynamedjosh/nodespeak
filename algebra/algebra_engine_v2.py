from functools import reduce
import math
from token import *

def tokenize(formula):
    formula = [unknown(formula)]
    operator_list = sorted(['+', '-', '*', '/', '^', '%', '(', ',', ')'], key=lambda e: -len(e))
    # Split on each occurance of an operator.
    for op in operator_list: # Check for the presence of each operator individually
        oldFormula = list(formula)
        formula = []
        for chunk in oldFormula:
            if(chunk.role == TokenRole.UNKNOWN): # Don't attempt to split operators.
                bits = chunk.label.split(op)
                for i, bit in enumerate(bits):
                    if(bit):
                        formula.append(unknown(bit.strip()))
                    if(i != len(bits) - 1):
                        if(op == ','):
                            pass # Don't include commas.
                        elif(op == '/'):
                            # Multiply by reciprocal
                            formula.append(operator('*'))
                            formula.append(operator('/'))
                        elif(op == '-'):
                            # Test to see if it is unary minus sign or an actual subtraction.
                            if((len(formula) == 0) or (formula[-1].role == TokenRole.OPERATOR)): # Unary -, it's ok
                                formula.append(operator('-'))
                            else: # Subtraction, change it to addition of negation.
                                formula.append(operator('+'))
                                formula.append(unknown('-1'))
                                formula.append(operator('*'))
                        else:
                            formula.append(operator(op))
            else:
                formula.append(chunk)
    print(formula)
    # Pack parenthesis / functions into function tokens, creating a structured hierarchy of the equation.
    stack = [function('', [])]
    for token in formula:
        current = stack[-1]
        if(token.label == '('):
            sub = function('', [])
            if((len(current.subs) > 0) and (current.subs[-1].role != TokenRole.OPERATOR)):
                sub.label = current.subs[-1].label
                current.subs = current.subs[:-1]
            stack.append(sub)
        elif(token.label == ')'):
            stack[-2].subs.append(stack[-1])
            stack = stack[:-1]
        else:
            if(token.role == TokenRole.UNKNOWN):
                try:
                    token = number(float(token.label))
                except:
                    token.role = TokenRole.VARIABLE
            stack[-1].subs.append(token)
    def opify_token(token):
        order_of_ops = ['^', '-', '/', '*', '+'] # Remember, - is unary negation, not subtraction.
        names = {'^': 'pow', '-': 'neg', '/': 'recip', '*': 'mul', '+': 'add'}
        for i, t in enumerate(token.subs):
            if(t.role == TokenRole.FUNCTION):
                opify_token(t)
                if(t.label == ''): # It was just some parenthesis
                    token.subs[i] = t.subs[0] # Get the actual content, remove the superfluous parenthesis (because everything has been converted to functions.
                else:
                    token.subs[i] = t
        [opify_token(t) for t in token.subs if t.role == TokenRole.FUNCTION]
        for op in order_of_ops:
            i = 0
            while i < len(token.subs):
                t = token.subs[i]
                if(t.label == op):
                    if(op in '^*+'): # Binary operators
                        funced = function(names[op], [token.subs[i-1], token.subs[i+1]])
                        if(op in '*+'): # Chain operators
                            if(funced.subs[0].label == names[op]):
                                funced.subs = funced.subs[0].subs + [funced.subs[1]]
                        token.subs = token.subs[:i-1] + [funced] + token.subs[i+2:]
                    else: # Unary operaors
                        funced = function(names[op], [token.subs[i+1]])
                        token.subs = token.subs[:i] + [funced] + token.subs[i+2:]
                        i += 1
                else:
                    i += 1
    opify_token(stack[0])
    return stack[0].subs[0]

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
    
    for i, t in enumerate(token.subs):
        if(t.role == TokenRole.FUNCTION):
            token.subs[i] = simplify_constants(t)
    constants = [t.numeric for t in token.subs if t.role == TokenRole.NUMBER]
    if(len(constants) == len(token.subs)): # All inputs are constant.
        return number(do_func(token.label, constants))
    elif(len(constants) > 1):
        args = [number(do_func(token.label, constants))]
        args += [t for t in token.subs if t.role != TokenRole.NUMBER]
        return function(token.label, args)
    else:
        return token

def simplify(token):
    if(len(token.subs) > 0):
        for i, t in enumerate(token.subs):
            token.subs[i] = simplify(t)
            
    ot = str(token)
    # (a^b)^c = a^(b*c)
    if(token.matches(
        TQ('func_pow', [
            TQ('func_pow')
        ], order_matters = True))):
        base = token.subs[0].subs[0]
        exps = function('mul', [token.subs[0].subs[1], token.subs[1]])
        token.subs = [base, exps]
    # (a^b) * (a^c) * ... * (a^z) = a^(b+c+...+z)
    if(token.matches(
        TQ('func_mul', [
            TQ('func_pow'),
            TQ('func_pow')
        ])
    )):
        pows = [i for i in token.subs if i.__name__() == 'func_pow']
        other = [i for i in token.subs if i.__name__() != 'func_pow']
        bases = {}
        for pow in pows:
            bases[str(pow.subs[0])] = []
        for i, pow in enumerate(pows):
            bases[str(pow.subs[0])].append(i)
        token = function('mul', other)
        for base in bases.keys():
            token.subs.append(function('pow', [
                pows[bases[base][0]].subs[0], 
                function('add', [pows[i].subs[1] for i in bases[base]])
            ]))
    # A^b*C^d = A^(b+(log(A)(C))d)
    if(token.matches(
        TQ('func_mul', [
            TQ('func_pow', [
                TQ('num_'),
                TQ('')
            ]),
            TQ('func_pow', [
                TQ('num_'),
                TQ('')
            ])
        ])
    )):
        pows = [i for i in token.subs if i.__name__() == 'func_pow']
        other = [i for i in token.subs if i.__name__() != 'func_pow']
        minb = min([pow.subs[0].numeric for pow in pows]) # Lowest of all the bases.
        # (log base [minb] via change of base formula)
        parts = [(math.log(pow.subs[0].numeric) / math.log(minb), pow.subs[1]) for pow in pows]
        pow_token = function('pow', [
            number(minb), 
            function('add', [
                function('mul', [
                    number(part[0]),
                    part[1]
                ])
                for part in parts
            ])
        ])
        if(len(other) == 0):
            token = pow_token
        else:
            token = function('mul', [pow_token] + other)
    '''
    # a^B = a * a * ..[B times].. * a
    if(token.matches(
        TQ('func_pow', [
            TQ('var_'),
            TQ('num_')
        ], order_matters=True)
    )):
        power = token.subs[1].subs
        if(int(power) == power): # It is an integer.
            if((power > 0) and (power < 16)): # Don't expand stuff with large exponents, for sanity's sake.
                token = function('mul', [variable(token.subs[0].subs) for i in range(int(power))])
            elif((power < 0) and (power > -16)):
                token = function('div', [number(1.0), function('mul', [variable(token.subs[0].name) for i in range(power)])])
    '''
    # a^0 = 1
    if(token.matches(
        TQ('func_pow', [
            TQ(''),
            TQ('num_0.0')
        ], order_matters=True)
    )):
        token = number(1.0)
    # a^1 = a
    if(token.matches(
        TQ('func_pow', [
            TQ(''),
            TQ('num_1.0')
        ], order_matters=True)
    )):
        token = token.subs[0]
    # a + 0 = a
    if(token.matches(TQ('func_add'))):
        parts = [t for t in token.subs if not t.matches(TQ('num_0.0'))]
        if(len(parts) == 0):
            token = number(0.0)
        elif(len(parts) == 1):
            token = parts[0]
        else:
            token = function('add', parts)
    # a * 1 = a
    # a * 0 = 0
    if(token.matches(TQ('func_mul'))):
        if(max([t.matches(TQ('num_0.0')) for t in token.subs])):
            token = number(0.0)
        else:
            parts = [t for t in token.subs if not t.matches(TQ('num_1.0'))]
            if(len(parts) == 0):
                token = number(1.0)
            elif(len(parts) == 1):
                token = parts[0]
            else:
                token = function('mul', parts)
            
    if(len(token.subs) > 0):
        for i, t in enumerate(token.subs):
            token.subs[i] = simplify(t)
    
    #print(ot, '->', token)
    return token
    
def parse_formula(formula):
    o = formula
    print(o)
    o = tokenize(o)
    print(o)
    o = simplify_constants(o)
    print(o)
    old = ''
    while str(o) != old:
        old = str(o)
        o = simplify(o)
        o = simplify_constants(o)
    print(o)
    return o

#print(function('Hello', [function('Hi', [])]))
parse_formula('2^b*4^d')
    