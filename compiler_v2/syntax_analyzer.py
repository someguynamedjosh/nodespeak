from context_free_grammars import ContextFreeGrammar, AbstractSyntaxTree
from constants import TokenType

# Everything prefixed by a $ is an elementary function. Everything prefixed by @ is a user-space function. Everything prefixed by # is a structural / compile-time function.

cfg = ContextFreeGrammar()
cfg.create_production(('ROOT',), ('SCOPE',))

cfg.create_production(('SCOPE',), ('SCOPE',), ('STAT',), remap=lambda e: e[0].children + e[1].children)
cfg.create_production(('SCOPE',))

cfg.create_production(('STAT',), ('EXP',), TokenType.SEMICOLON, remap=lambda e: e[0].children)
cfg.create_production(('STAT',), TokenType.OPEN_BRACE, ('STATS',), TokenType.CLOSE_BRACE)
cfg.create_production(('STAT',), TokenType.IDENTIFIER, TokenType.ASSIGNMENT_OPERATOR, ('EXP',), TokenType.SEMICOLON, remap=lambda e:
                      [AbstractSyntaxTree('#assign', e[0], e[2].children[0])])
cfg.create_production(
    ('SCOPE',), TokenType.KEYWORD_RETURN, ('EXP',), TokenType.SEMICOLON, remap=lambda e:
    [AbstractSyntaxTree('#assign', (TokenType.IDENTIFIER, 'return'), e[1].children[0]), AbstractSyntaxTree('#return')])

cfg.create_production(('EXP',), TokenType.OPEN_PAREN, ('EXP',), TokenType.CLOSE_PAREN, remap=lambda e: e[1].children)
cfg.create_production(('EXP',), TokenType.NUMBER)
cfg.create_production(('EXP',), TokenType.OCTAL_NUMBER)
cfg.create_production(('EXP',), TokenType.BINARY_NUMBER)
cfg.create_production(('EXP',), TokenType.HEX_NUMBER)
cfg.create_production(('EXP',), TokenType.IDENTIFIER)#, remap=lambda e: [AbstractSyntaxTree('$get', e[0])])

# The remap rules for each production are NOT designed to be computationally efficient. They
# are instead designed to provide a simplified algebraic representation of computations using
# only a handful of operators, which lowers the amount of work the algebraic optimization step
# will have to do.
cfg.create_binary_operator_production(
    ('EXP',), TokenType.DOT_OPERATOR, tier=0, remap=lambda e: 
    [AbstractSyntaxTree('#access', e[0].children[0], e[2].children[0])])
cfg.create_production(('ARRAY_ACCESS',), ('EXP',), TokenType.OPEN_BRACKET, ('EXP',), TokenType.CLOSE_BRACKET)
cfg.create_production(('EXP',), ('ARRAY_ACCESS',), remap=lambda e: [AbstractSyntaxTree('#index', e[0].children[0], e[0].children[2])])
def mul_remap(e):
    if(e[1][1] == '/'):
        return [AbstractSyntaxTree('$mul', e[0].children[0], AbstractSyntaxTree('$recip', e[2].children[0]))]
    else:
        return [AbstractSyntaxTree({'*': '$mul', '%': '$mod'}[e[1][1]], e[0].children[0], e[2].children[0])]
cfg.create_binary_operator_production(('EXP',), TokenType.MULTIPLY_OPERATOR, tier=1, remap=mul_remap)
cfg.create_binary_operator_production(
    ('EXP',), TokenType.MINUS_OPERATOR, tier=2, remap=lambda e:
    [AbstractSyntaxTree('$add', e[0].children[0], AbstractSyntaxTree('$mul',  e[2].children[0], (TokenType.NUMBER, '-1')))])
cfg.create_binary_operator_production(
    ('EXP',), TokenType.ADD_OPERATOR, tier=2, remap=lambda e:
    [AbstractSyntaxTree('$add', e[0].children[0], e[2].children[0])])
cfg.create_binary_operator_production(
    ('EXP',), TokenType.BITWISE_OPERATOR, tier=3, remap=lambda e:
    [AbstractSyntaxTree({'~@': '$bxor', '~|': '$bor', '~&': '$band', '~!=': '$bne', '~==': '$beq'}[e[1][1]], e[0].children[0], e[2].children[0])])
cfg.create_binary_operator_production(
    ('EXP',), TokenType.COMPARISON_OPERATOR, tier=4, remap=lambda e:
    [AbstractSyntaxTree({'!=': '$neq', '==': '$eq', '<=': '$leq', '>=': '$geq', '<': '$lst', '>': '$grt'}[e[1][1]], e[0].children[0], e[2].children[0])])
cfg.create_binary_operator_production(
    ('EXP',), TokenType.BOOLEAN_OPERATOR, tier=5, remap=lambda e:
    [AbstractSyntaxTree({'@': '$xor', '|': '$or', '&': '$and'}[e[1][1]], e[0].children[0], e[2].children[0])])

cfg.create_production(('EXP',), TokenType.NOT_OPERATOR, ('EXP',), remap=lambda e: [AbstractSyntaxTree('$not', e[1].children[0])])
cfg.create_production(('EXP',), TokenType.BITWISE_NOT_OPERATOR, ('EXP',), remap=lambda e: [AbstractSyntaxTree('$bnot', e[1].children[0])])

cfg.create_production(('NB_IN_LIST',), ('IN_LIST',))
cfg.create_production(('NB_IN_LIST',), remap=lambda e: [AbstractSyntaxTree(('IN_LIST',))])
cfg.create_production(('IN_LIST',), ('IN_LIST',), TokenType.COMMA, ('INPUT',), remap=lambda e: e[0].children + e[2].children)
cfg.create_production(('IN_LIST',), ('INPUT',), remap=lambda e: e[0].children)
cfg.create_production(('INPUT',), ('EXP',), remap=lambda e: e[0].children)

cfg.create_production(('NB_OUT_LIST',), ('OUT_LIST',))
cfg.create_production(('NB_OUT_LIST',), remap=lambda e: [AbstractSyntaxTree(('OUT_LIST',))])
cfg.create_production(('OUT_LIST',), ('OUT_LIST',), TokenType.COMMA, ('OUTPUT',), remap=lambda e: e[0].children + e[2].children)
cfg.create_production(('OUT_LIST',), ('OUTPUT',), remap=lambda e: e[0].children)
cfg.create_production(('OUTPUT',), TokenType.IDENTIFIER, TokenType.IDENTIFIER, remap=lambda e: [AbstractSyntaxTree('#decvar', e[0], e[1])]) # For defining new variables inside of the output list.
cfg.create_production(('OUTPUT',), TokenType.IDENTIFIER)
cfg.create_production(('OUTPUT',), TokenType.KEYWORD_RETURN)

cfg.create_production(
    ('EXP',), TokenType.IDENTIFIER, TokenType.OPEN_PAREN, ('NB_IN_LIST',), TokenType.CLOSE_PAREN, remap=lambda e:
    [AbstractSyntaxTree('@' + e[0][1], e[2].children[0])])
cfg.create_production(
    ('EXP',), TokenType.IDENTIFIER, TokenType.OPEN_PAREN, ('NB_IN_LIST',), TokenType.CLOSE_PAREN, TokenType.COLON,
    TokenType.OPEN_PAREN, ('NB_OUT_LIST',), TokenType.CLOSE_PAREN, remap=lambda e:
    [AbstractSyntaxTree('@' + e[0][1], e[2].children[0], e[6].children[0])])

cfg.create_production( # For defining variables with basic data types.
    ('STAT',), TokenType.IDENTIFIER, ('VAR_DEC_LIST',), TokenType.SEMICOLON, remap=lambda e: 
    [AbstractSyntaxTree('#decvar', e[0], *i.children) for i in e[1].children])
cfg.create_production( # For defining variables with array data types.
    ('STAT',), ('ARRAY_ACCESS',), ('VAR_DEC_LIST',), TokenType.SEMICOLON, remap=lambda e: 
    [AbstractSyntaxTree('#decvar', AbstractSyntaxTree('#arrayType', e[0].children[0].children[0], e[0].children[2]), *i.children) for i in e[1].children])
cfg.create_production(('VAR_DEC_LIST',), ('VAR_DEC_LIST',), TokenType.COMMA, ('VAR_DEC',), remap=lambda e: e[0].children + e[2].children)
cfg.create_production(('VAR_DEC_LIST',), ('VAR_DEC',), remap=lambda e: e[0].children)
cfg.create_production(('VAR_DEC',), TokenType.IDENTIFIER, remap=lambda e: [AbstractSyntaxTree('#decvar', e[0][1])])
cfg.create_production(('VAR_DEC',), TokenType.IDENTIFIER, TokenType.ASSIGNMENT_OPERATOR, ('EXP',), remap=lambda e: [AbstractSyntaxTree('#decvar', e[0], e[2].children[0])])

cfg.create_production(('IN_LIST_DEF',), ('IN_LIST_DEF',), TokenType.COMMA, ('IN_DEF',), remap=lambda e: e[0].children + e[2].children)
cfg.create_production(('IN_LIST_DEF',), ('IN_DEF',), remap=lambda e: e[0].children)
cfg.create_production(('IN_DEF',), TokenType.IDENTIFIER, TokenType.IDENTIFIER, remap=lambda e: [AbstractSyntaxTree('#decvar', e[0], e[1])]) # E.G. Int a
cfg.create_production(
    ('IN_DEF',), TokenType.IDENTIFIER, TokenType.IDENTIFIER, TokenType.ASSIGNMENT_OPERATOR, ('EXP',), remap=lambda e: 
    [AbstractSyntaxTree('#decvar', e[0], e[1], e[3].children[0])]) # E.G. Int a = 2

cfg.create_production(('OUT_LIST_DEF',), ('OUT_LIST_DEF',), TokenType.COMMA, ('OUT_DEF',), remap=lambda e: e[0].children + e[2].children)
cfg.create_production(('OUT_LIST_DEF',), ('OUT_DEF',), remap=lambda e: e[0].children)
cfg.create_production(('OUT_DEF',), TokenType.IDENTIFIER, TokenType.IDENTIFIER, remap=lambda e: [AbstractSyntaxTree('#decvar', e[0], e[1])]) # E.G. Int a
cfg.create_production(('OUT_DEF',), TokenType.IDENTIFIER, TokenType.KEYWORD_RETURN, remap=lambda e: [AbstractSyntaxTree('#decvar', e[0], (TokenType.IDENTIFIER, 'return'))]) # E.G. Int return

cfg.create_production(
    ('STAT',), TokenType.KEYWORD_DEF, TokenType.IDENTIFIER, TokenType.OPEN_PAREN, ('IN_LIST_DEF',), TokenType.CLOSE_PAREN, TokenType.OPEN_BRACE, 
    ('SCOPE',), TokenType.CLOSE_BRACE, remap=lambda e: 
    [AbstractSyntaxTree('#decfunc', e[1], e[3], AbstractSyntaxTree(('OUT_LIST_DEF',)), e[6])])
cfg.create_production(
    ('STAT',), TokenType.KEYWORD_DEF, TokenType.IDENTIFIER, TokenType.OPEN_PAREN, ('IN_LIST_DEF',), TokenType.CLOSE_PAREN, TokenType.COLON, 
    TokenType.IDENTIFIER, TokenType.OPEN_BRACE, ('SCOPE',), TokenType.CLOSE_BRACE,
    remap=lambda e: [AbstractSyntaxTree('#decfunc', e[1], e[3], 
                                        AbstractSyntaxTree(('OUT_LIST_DEF',), 
                                                           AbstractSyntaxTree('#decvar', e[6], (TokenType.IDENTIFIER, 'return'))), 
                                        e[8])])
cfg.create_production(
    ('STAT',), TokenType.KEYWORD_DEF, TokenType.IDENTIFIER, TokenType.OPEN_PAREN, ('IN_LIST_DEF',), TokenType.CLOSE_PAREN, TokenType.COLON,
    TokenType.OPEN_PAREN, ('OUT_LIST_DEF',), TokenType.CLOSE_PAREN, TokenType.OPEN_BRACE, ('SCOPE',), TokenType.CLOSE_BRACE, 
    remap=lambda e: [AbstractSyntaxTree('#decfunc', e[1], e[3], e[7], e[10])])

cfg.create_production(('ELIF',), TokenType.KEYWORD_ELIF, ('EXP',), TokenType.OPEN_BRACE, ('SCOPE',), TokenType.CLOSE_BRACE, remap=lambda e:
                      [AbstractSyntaxTree('$branch', e[1], e[3])])
cfg.create_production(('ELIFS',), ('ELIF',), ('ELIFS',), remap=lambda e: [AbstractSyntaxTree('$branch', *e[0].children[0].children, AbstractSyntaxTree(('SCOPE',), e[1].children[0]))])
cfg.create_production(('ELIFS',), ('ELIF',), remap=lambda e: e[0].children)
cfg.create_production(('ELIFS',), TokenType.KEYWORD_ELSE, TokenType.OPEN_BRACE, ('SCOPE',), TokenType.CLOSE_BRACE, remap=lambda e: e[2].children)
cfg.create_production(('STAT',), TokenType.KEYWORD_IF, ('EXP',), TokenType.OPEN_BRACE, ('SCOPE',), TokenType.CLOSE_BRACE, ('ELIFS',), remap=lambda e:
                      [AbstractSyntaxTree('$branch', e[1], e[3], AbstractSyntaxTree(('SCOPE',), *e[5].children))])

cfg.create_production(('ARRAY_ITEMS',), ('EXP',))
cfg.create_production(('ARRAY_ITEMS',), ('ARRAY_ITEMS',), TokenType.COMMA, ('EXP',), remap=lambda e: e[0].children + [e[2]])
cfg.create_production(('ARRAY_LITERAL',), TokenType.OPEN_BRACKET, ('ARRAY_ITEMS',), TokenType.CLOSE_BRACKET, remap=lambda e: e[1].children)
cfg.create_production(('EXP',), ('ARRAY_LITERAL',), remap=lambda e: [AbstractSyntaxTree('#arrayLiteral', *e[0].children)])

debug = False
debug and print(cfg)
#print(cfg.follow(('EXP',)))
cfgfa = cfg.create_dfa(('ROOT',), eof_symbol=TokenType.EOF, debug=debug)

def build_ast(tokens):
    return cfgfa.test(tokens, lambda e: e[0]) # The CFGFA should change states based on the token type, not based on the entire token.

if __name__ == '__main__':
    print(cfgfa)
    print(cfg.follow(('EXP',)))