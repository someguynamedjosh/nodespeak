from mini_regexp import r_and, r_or, r_repeat, regexp_to_nfa, SET_LETTERS, SET_NUMBERS, SET_ALPHANUMERIC, gen_range
from finite_automata import NondeterminateFiniteAutomaton
from constants import TokenType
T = TokenType

res = {}

res[T.NUMBER] = r_and( # ((([0-9]+(\.[0-9]*)?)|(\.[0-9]+))([eE][+-]?[0-9]+)?)
    r_or( # a(\.?(b)?) or \.b
        r_and( # a[.[b]]
               r_repeat(r_or(SET_NUMBERS), 1),
               r_repeat(
                   r_and(
                       '.',
                       r_repeat(r_or(SET_NUMBERS)) # 0 or more numbers
                   ), 0, 1
               )
        ),
        r_and(
           '.',
           r_repeat(r_or(SET_NUMBERS), 1) # 1 or more numbers
        )
    ), r_repeat(
        r_and( # [eE][+-]?[0-9]+
            r_or('eE'),
            r_repeat(r_or('+-'), 0, 1),
            r_repeat(r_or(SET_NUMBERS), 1)
        ), 0, 1
    )
)

res[T.HEX_NUMBER] = r_and( # 0x[0-9a-fA-F]+
    '0x',
    r_repeat(
        r_or(
            SET_NUMBERS + gen_range('a', 'f') + gen_range('A', 'F')
        ), 1
    )
)

res[T.BINARY_NUMBER] = r_and( # 0b(0|1)+
    '0b',
    r_repeat(
        r_or('01'), 1
    )
)

res[T.OCTAL_NUMBER] = r_and( # 0[0-7]+
    '0',
    r_repeat(
        r_or(SET_NUMBERS), 1
    )
)

res[T.IDENTIFIER] = r_and( # [a-zA-Z][a-zA-Z0-9_]*
    r_or(SET_LETTERS),
    r_repeat(
        r_or(SET_LETTERS + SET_NUMBERS + ['_'])
    )
)

res[T.MULTIPLY_OPERATOR] = r_or('*', '/', '%')
res[T.ADD_OPERATOR] = r_or('+')
res[T.MINUS_OPERATOR] = r_and('-')
arithmetic_operators = ['%', '/', '*', '+', '-']

boolean_operators = ['@', '|', '&'] # Only apply to two booleans.
res[T.BOOLEAN_OPERATOR] = r_or([r_and(i) for i in boolean_operators])
comparison_operators = ['!=', '==', '<=', '>=', '<', '>'] # Only apply to two numbers.
res[T.COMPARISON_OPERATOR] = r_or([r_and(i) for i in comparison_operators])
res[T.NOT_OPERATOR] = r_and('!') # Only applies to one boolean
res[T.BITWISE_NOT_OPERATOR] = r_and('~!') # Only applies to one number.
bitwise_operators = [i + '~' for i in boolean_operators + ['!=', '==']]
res[T.BITWISE_OPERATOR] = r_or([r_and(i) for i in bitwise_operators]) # Only applies to two numbers.

# a = b; a [arithmetic operator]= b (equivalent to a = a [arithmetic operator] b); 
# a [bitwise operator]= b (equivalent to a = a [bitwise operator] b); 
# a [logical operator]= b (very restricted set of logical operators, because others require numbers and return booleans.)
assignment_operators = [i + '=' for i in arithmetic_operators] + [i + '=' for i in bitwise_operators] + [i + '=' for i in boolean_operators]
res[T.ASSIGN_WITH_MATH_OPERATOR] = r_or([r_and(i) for i in assignment_operators])
res[T.ASSIGNMENT_OPERATOR] = r_and('=')

res[T.DOT_OPERATOR] = r_and('.')
res[T.COLON] = r_and(':')
res[T.SEMICOLON] = r_and(';')
res[T.COMMA] = r_and(',')
res[T.OPEN_PAREN] = r_and('(')
res[T.CLOSE_PAREN] = r_and(')')
res[T.OPEN_BRACE] = r_and('{')
res[T.CLOSE_BRACE] = r_and('}')
res[T.OPEN_BRACKET] = r_and('[')
res[T.CLOSE_BRACKET] = r_and(']')
 
ignore = [' ', '\t', '\n', '\r']
ignore += [r_and('#', r_repeat(r_or(gen_range(' ', '~'))),'\n')] # Any printable characters between # and a newline.
#ignore += [r_and('/*', r_repeat(r_or(gen_range('\x01', '\xff'))),'*/')] # Any characters between /* and */
res[T.IGNORE] = r_or(*ignore)

res[T.KEYWORD_IF] = r_and('if')
res[T.KEYWORD_ELIF] = r_and('elif')
res[T.KEYWORD_ELSE] = r_and('else')
res[T.KEYWORD_FOR] = r_and('for')
res[T.KEYWORD_WHILE] = r_and('while')
res[T.KEYWORD_DO] = r_and('do')
res[T.KEYWORD_TYPEDEF] = r_and('typedef')
res[T.KEYWORD_BREAK] = r_and('break')
res[T.KEYWORD_RETURN] = r_and('return')
res[T.KEYWORD_DEF] = r_and('def')

master_nfa = NondeterminateFiniteAutomaton()
start = master_nfa.add_state()
master_nfa.start_state = start
for nfa in [regexp_to_nfa(regexp, key.value) for key, regexp in res.items()]:
    offset = master_nfa.add_states(nfa)
    master_nfa.add_transition(start, '\x00', nfa.start_state + offset)
master_dfa = master_nfa.convert_to_dfa()

def convert_to_tokens(textual_program):
    tokens = []
    index = 0
    master_dfa.reset_run()
    for i, char in enumerate(textual_program):
        try:
            master_dfa.step(char)
        except:
            if(T.IGNORE.value not in master_dfa.get_current_states()): # Do not include whitespace in the outputted tokens.
                tokens.append((T(max(master_dfa.get_current_states())), textual_program[index:i]))
            master_dfa.reset_run()
            master_dfa.step(char)
            index = i
    tokens.append((T(max(master_dfa.get_current_states())), textual_program[index:]))
    if(tokens[-1][0] == T.IGNORE): # If we accidentally added a space, newline, or tab...
        tokens = tokens[:-1]
    return tokens + [(TokenType.EOF, '\x00')]

if __name__ == '__main__':
    print(master_dfa)
    program = 'int a = 0x123AFa + 2 + 3;'
    tokens = convert_to_tokens(program)
    print(tokens)
