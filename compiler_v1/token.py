from enum import Enum

class TokenQuery:
    def __init__(self, pattern, sub_queries=None, order_matters=False, quantity_matters=False):
        self.pattern = pattern
        if(sub_queries):
            self.sub_queries = sub_queries
        else:
            self.sub_queries = []
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

class TokenRole(Enum):
    UNKNOWN = 0
    OPERATOR = 1
    NUMBER = 2
    VARIABLE = 3
    FUNCTION = 4
    TRANSFORM = 10
    STATEMENT = 5
    #LINES = 6
    NAMESPACE = 7
    GROUP= 8
    STRING = 9
    DATATYPE = 11
    OBJ_DATATYPE = 12 

class Token:
    def __init__(self, role, label, subs=[], numeric=None):
        self.role = role
        self.label = label
        if(subs):
            self.subs = subs
        else:
            self.subs = []
        self.numeric = numeric
    
    def __name__(self):
        return ['', 'op_', 'num_', 'var_', 'func_', 'statement_', '', 'namespace_', 'group_', 'str_', 'trans_', 'datatype_', 'objDatatype_'][self.role.value] + self.label
    
    def matches(self, pattern):
        n = self.__name__()
        if(len(n) < len(pattern.pattern)):
            return False
        elif(self.__name__()[:len(pattern.pattern)] != pattern.pattern):
            return False
        elif(pattern.check_args()):
            if(self.role != TokenRole.FUNCTION):
                return False
            if((pattern.quantity_matters) and (len(pattern.sub_queries) != len(self.subs))):
                return False
            if(pattern.order_matters):
                for i, sq in enumerate(pattern.sub_queries):
                    if(not self.subs[i].matches(sq)):
                        return False
                return True
            else:
                used = []
                for sq in pattern.sub_queries:
                    for i, arg in enumerate(self.subs):
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
    
    def __str__(self):
        return self.__repr__()
    
    def __repr__(self):
        if(self.role in [TokenRole.FUNCTION, TokenRole.GROUP, TokenRole.NAMESPACE, TokenRole.STATEMENT, TokenRole.TRANSFORM, TokenRole.OBJ_DATATYPE]):
            return self.__name__() + ' {' + ', '.join([''.join([
                    '\n\t' + line 
                    for line in str(arg).split('\n')
                ]) for arg in self.subs
            ]) + '\n}'
        elif(self.role == TokenRole.STRING):
            return '"' + self.label + '"'
        else:
            return self.__name__()

def unknown(label):
    return Token(TokenRole.UNKNOWN, label)

def operator(label):
    return Token(TokenRole.OPERATOR, label)

def function(label, contents):
    return Token(TokenRole.FUNCTION, label, contents)

def variable(label):
    return Token(TokenRole.VARIABLE, label)

def number(num):
    return Token(TokenRole.NUMBER, str(num), numeric=num)

def statement(name, contents):
    return Token(TokenRole.STATEMENT, name, contents)

def namespace(name, contents=None):
    return Token(TokenRole.NAMESPACE, name, contents)

def group(delimiter='(', contents=None):
    return Token(TokenRole.GROUP, delimiter, contents)

def string(contents):
    return Token(TokenRole.STRING, contents)

def transform(label, contents):
    return Token(TokenRole.TRANSFORM, label, contents)

def datatype(label):
    return Token(TokenRole.DATATYPE, label)

def obj_datatype(contents=[]):
    return Token(TokenRole.OBJ_DATATYPE, '', contents)
