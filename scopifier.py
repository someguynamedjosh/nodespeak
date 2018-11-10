from token import *
from errors import *
from scope import *

def convert_type(parent_scope, token):
    if(token.label == '{'):
        members = [DefaultableVariable(convert_type(parent_scope, sub.subs[0]), sub.subs[1].label) for sub in token.subs if sub.__name__() == 'statement_defineVar']
        for sub in [sub for sub in token.subs if sub.__name__() == 'statement_assign']:
            for i, var in enumerate(members):
                if(sub.subs[0] == var.name):
                    var.default_value = sub.subs[1]
        return ObjectDatatype(members)
    elif(token.__name__() == 'func_*arrayType'):
        return ArrayDatatype(convert_type(parent_scope, token.subs[0]), token.subs[1].numeric)
    else:
        return parent_scope.find_datatype(token.label)

def create_root_scope():
    tr = Scope()
    tr.add_datatype(Datatype('Int', 4))
    tr.add_datatype(Datatype('Float', 4))
    return tr

def process(token, parent_scope=None):
    if(not parent_scope):
        parent_scope = create_root_scope()
    if(token.role == TokenRole.NAMESPACE):
        scope = Scope(parent=parent_scope)
        for sub in token.subs:
            if(sub.__name__() == 'statement_typedef'):
                scope.add_datatype(convert_type(scope, sub.subs[0]))
            elif(sub.__name__() == 'statement_defineVar'):
                scope.add_variable(Variable(convert_type(scope, sub.subs[0]), sub.subs[1].label))
            elif(sub.__name__() in ['statement_defineTrans', 'statement_defineFunc']):
                seperator = 0
                for i in range(len(sub.subs)):
                    if(sub.subs[i].__name__() == 'op_:'):
                        seperator = i
                        break
                ins = [DefaultableVariable(convert_type(scope, i.subs[0]), i.subs[1].label) for i in sub.subs[1:seperator]]
                outs = [DefaultableVariable(convert_type(scope, i.subs[0]), i.subs[1].label) for i in sub.subs[seperator+1:-1]]
                scope.add_function(Function(sub.subs[0].label, ins, outs, process(sub.subs[-1], scope)))
            else:
                if(sub.__name__() == 'statement_branch'):
                    for i, ssub in enumerate(sub.subs):
                        if(ssub.role == TokenRole.NAMESPACE):
                            sub.subs[i] = process(ssub, scope)
                scope.add_statement(sub)
        return scope
                