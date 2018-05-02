from token import *
from errors import *
from operator import sub

def apply_pass(pass_func, input_token):
    for i, sub in enumerate(input_token.subs):
        if(len(sub.subs) > 0):
            input_token.subs[i] = pass_func(sub)

# Replaces name() with actual function calls and name():() with transform calls and name():datatype with placeholder function calls
def pass05(input_token):
    apply_pass(pass05, input_token)
    subs = input_token.subs
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'group_('): # Starting a function / transform call
            if(len(output) == 0 or output[-1].role == TokenRole.OPERATOR):
                output.append(sub)
            else:
                output[-1] = function(output[-1].label, [sub])   
        elif(n == 'op_:'): # It is a transform call or a function definition with a return type.
            output[-1].subs.append(subs[i+1])
            i += 1
        else:
            output.append(sub)
        i += 1
        
    input_token.subs = output
    return input_token                

# Replace [] and . with *arrayAccess and *memberAccess
def pass10(input_token):
    apply_pass(pass10, input_token)
    subs = input_token.subs
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'group_['):
            # If this is true, it's probably an array literal rather than an array access.
            if(len(output) == 0 or output[-1].role == TokenRole.OPERATOR):
                output.append(sub)
            else:
                sub.label = '('
                output[-1] = function('*arrayAccess', [output[-1], sub])
        elif(n == 'operator_.'):
            output[-1] = function('*memberAccess', [output[-1], string(subs[i+1].label)])
            i += 1 # Skip over i+1
        else:
            output.append(sub)
        i += 1
        
    input_token.subs = output
    return input_token

# Replaces / with #reciprocal and - with #multiply
def pass20(input_token):
    apply_pass(pass20, input_token)
    subs = input_token.subs
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'op_/'):
            output.append(operator('*'))
            output.append(function('#reciprocal', [subs[i+1]]))
            i += 1
        elif(n == 'op_-'):
            if(i != 0 and output[i-1].role != TokenRole.OPERATOR): # Binary minus
                output.append(operator('+'))
            output.append(function('#multiply', [subs[i+1], number(-1.0)]))
            i += 1
        else:
            output.append(sub)
        i += 1
    
    input_token.subs = output
    return input_token

# Replaces ! with #not, then ^ with #power, then * with #multiply, then % with #modulo, then + with #add 
def pass30(input_token):
    apply_pass(pass30, input_token)
    subs = input_token.subs
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'op_!'):
            output.append(function('#not', [subs[i+1]]))
            i += 1
        else:
            output.append(sub)
        i += 1
    
    subs = output
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'op_^'):
            output[-1] = function('#power', [subs[i-1], subs[i+1]])
            i += 1
        else:
            output.append(sub)
        i += 1
    
    subs = output
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'op_*'):
            if(output[-1].__name__() == 'func_#multiply'):
                output[-1].subs.append(subs[i+1])
            else:
                output[-1] = function('#multiply', [subs[i-1], subs[i+1]])
            i += 1
        else:
            output.append(sub)
        i += 1
    
    subs = output
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'op_%'):
            output[-1] = function('#modulo', [subs[i-1], subs[i+1]])
            i += 1
        else:
            output.append(sub)
        i += 1
    
    subs = output
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'op_+'):
            if(output[-1].__name__() == 'func_#add'):
                output[-1].subs.append(subs[i+1])
            else:
                output[-1] = function('#add', [subs[i-1], subs[i+1]])
            i += 1
        else:
            output.append(sub)
        i += 1
    
    input_token.subs = output
    return input_token

# Replaces < > <= >= == != & | @ with their # versions, starting with the bitwise versions and then the regular versions.
def pass40(input_token):
    apply_pass(pass40, input_token)
    subs = input_token.subs
    output = subs
    
    for op in ['<#less', '>#greater', '<=#lessOrEqual', '>=#greaterOrEqual', '==#equal', '!=#notEqual', '&#and', '|#or', '@#xor']:
        subs = output
        output = []
        i = 0
        while(i < len(subs)):
            sub = subs[i]
            n = sub.__name__()
            if(n == 'op_' + op.split('#')[0] + '~'):
                output[-1] = function('#' + op.split('#')[1] + 'Bitwise', [subs[i-1], subs[i+1]])
                i += 1
            else:
                output.append(sub)
            i += 1
    
    for op in ['<#less', '>#greater', '<=#lessOrEqual', '>=#greaterOrEqual', '==#equal', '!=#notEqual', '&#and', '|#or', '@#xor']:
        subs = output
        output = []
        i = 0
        while(i < len(subs)):
            sub = subs[i]
            n = sub.__name__()
            if(n == 'op_' + op.split('#')[0]):
                output[-1] = function('#' + op.split('#')[1], [subs[i-1], subs[i+1]])
                i += 1
            else:
                output.append(sub)
            i += 1
    
    input_token.subs = output
    return input_token    

# Replaces if(){}, if(){}else{}, if(){}elif(){}, and if(){}elif(){}else{} with *branch
def pass50(input_token):  
    apply_pass(pass50, input_token)
    subs = input_token.subs
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'func_if'):
            output.append(statement('branch', [sub.subs[0], namespace('branch', subs[i+1].subs)]))
            i += 1
        elif(n == 'func_elif'):
            output[-1].subs += [sub.subs[0], namespace('branch', subs[i+1].subs)]
            i += 1
        elif(n == 'else'):
            output[-1].subs += [number(1.0), namespace('branch', subs[i+1].subs)]
            i += 1
        elif(n == 'group_{'):
            if(len(output) > 0 and output[-1].role == TokenRole.FUNCTION):
                name = 'defineFunc'
                if(len(output[-1].subs) > 1 and len(output[-1].subs[-1].subs) > 1):
                    name = 'defineTrans'
                output[-1] = statement(name, [string(output[-1].label)] + output[-1].subs + [namespace('code', sub.subs)])
            else:
                output.append(sub)
        else:
            output.append(sub)
        i += 1
        
    input_token.subs = output
    return input_token

# Replaces = with *assign
def pass60(input_token):
    apply_pass(pass60, input_token)
    subs = input_token.subs
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'op_='):
            output[-1] = statement('assign', [subs[i-1], subs[i+1]])
            i += 1
        else:
            output.append(sub)
        i += 1
        
    input_token.subs = output
    return input_token

# Things like array size declarations will be mislabeled as array access, this method changes that.
def typify(input_token):
    n = input_token.__name__()
    if(n == 'func_*arrayAccess'):
        input_token.label = '*arrayType'
        input_token.subs[0] = typify(input_token.subs[0])
    return input_token

# Replaces typedef type typename; with statement_typedef(type, typename)
# Also replaces return expression; with statement_assign(var_*returnValue, expression); statement_return();
def pass70(input_token):
    apply_pass(pass70, input_token)
    subs = input_token.subs
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'typedef'):
            output.append(statement('typedef', [typify(subs[i+1]), subs[i+2]]))
            i += 3
        elif(n == 'return'):
            if(len(subs) <= i+1 or subs[i+1].label == ','):
                output.append(sub)
            elif(subs[i+1].label == ';'):
                output.append(statement('return', []));
            else:
                output.append(statement('assign', [string('*returnValue'), subs[i+1]]))
                output.append(statement('return', []))
                i += 1
        else:
            output.append(sub)
        i += 1
        
    input_token.subs = output
    return input_token  

# Replaces datatype varame1, statement_assign(varname2, value) with statement_defineVar(datatype, varname1), statement_defineVar(datatype, varname2), statement_assign(varname2, value)
def pass80(input_token):
    apply_pass(pass80, input_token)
    # Only apply this pass to code blocks, argument lists, or object type definitions
    if(not (input_token.role == TokenRole.NAMESPACE or (input_token.__name__() in ['group_{', 'group_(']))):
        return input_token
    subs = input_token.subs
    output = []
    i = 0
    state = 0 # Looking for datatype
    datatype = None
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(state == 0): # Look for datatype
            if(sub.role == TokenRole.UNKNOWN and sub.label != 'typedef'):
                datatype = sub
                state = 1 # Look for variable name or variable default assignment.
            elif(n == 'func_*arrayAccess'):
                datatype = typify(sub)
                state = 1
            else:
                output.append(sub)
        elif(state == 1): # Look for variable name or variable default assignment.
            if(sub.role == TokenRole.UNKNOWN):
                output.append(statement('defineVar', [datatype, string(sub.label)]))
                state = 2 # Look for comma or semicolon
            elif(n == 'statement_assign'):
                output.append(statement('defineVar', [datatype, string(sub.subs[0].label)]))
                output.append(sub)
                state = 2 # Look for comma or semicolon
            elif(n == 'op_,'): # The datatype was just a variable.
                if(subs[i-1].role == TokenRole.UNKNOWN):
                    output.append(variable(subs[i-1].label))
                else:
                    output.append(subs[i-1])
                state = 0
        elif(state == 2): # Look for comma or semicolon
            if(sub.label == ','):
                # It is in a block of code or an object type, so comma means more variables with same datatype.
                if(input_token.role == TokenRole.NAMESPACE or input_token.__name__() == 'group_{'):
                    state = 1 # Look for variable name or variable default assignment.
                elif(input_token.__name__() == 'group_('): # It is an argument list, so comma means a new datatype is needed.
                    state = 0 # Look for datatype
            elif(sub.label == ';'):
                state = 0 # Look for another line of variable definitions.
        else:
            output.append(sub)
        i += 1
    if(state == 1 and datatype):
        output.append(variable(datatype.label))
        
    input_token.subs = output
    return input_token

# Collapses argument list of user-made functions and transforms to single-depth lists
def pass85(input_token):
    apply_pass(pass85, input_token)
    subs = input_token.subs
    if(input_token.role == TokenRole.FUNCTION):
        if(len(subs) == 1 and subs[0].__name__() == 'group_('):
            subs = subs[0].subs
        elif(len(subs) == 2 and subs[0].__name__() == 'group_(' and subs[1].__name__() == 'group_('):
            subs = subs[0].subs + [operator(':')] + subs[1].subs
    elif(input_token.__name__() == 'statement_defineFunc'):
        if(len(subs) < 4): # No return type specified.
            subs = [subs[0]] + subs[1].subs + [operator(':'), statement('defineVar', [unknown('Void'), string('*returnValue')]), subs[2]]
        else:
            subs = [subs[0]] + subs[1].subs + [operator(':'), statement('defineVar', [subs[2], string('*returnValue')]), subs[3]]
    elif(input_token.__name__() == 'statement_defineTrans'):
        subs = [subs[0]] + subs[1].subs + [operator(':')] + subs[2].subs + [subs[3]]
                
    input_token.subs = subs
    return input_token      

# Slightly simplify expressions by removing redundant parenthesis as well as changing unknown text tokens to variable tokens
# Also removes commas and semicolons.
def pass90(input_token):
    apply_pass(pass90, input_token)
    subs = input_token.subs
    output = []
    i = 0
    while(i < len(subs)):
        sub = subs[i]
        n = sub.__name__()
        if(n == 'group_('):
            # Parenthesis inside of function groups dictate inputs and outputs, do not collapse them.
            if(len(sub.subs) == 1):
                output.append(sub.subs[0])
            else:
                output.append(sub)
        elif(sub.role == TokenRole.UNKNOWN and (input_token.role == TokenRole.FUNCTION or input_token.__name__() in ['group_(', 'group_[', 'statement_assign'])):
            output.append(variable(sub.label))
        elif(sub.__name__() in ['op_;', 'op_,']): # Skip these.
            pass
        else:
            output.append(sub)
        i += 1
        
    input_token.subs = output
    return input_token

def process(code):
    passes = [pass05, pass10, pass20, pass30, pass40, pass50, pass60, pass70, pass80, pass85, pass90]
    for p in passes:
        #print(code)
        code = p(code)
        #print(p)
    return code
