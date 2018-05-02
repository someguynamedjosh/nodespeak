from token import TokenRole, operator, variable, unknown, string, statement,\
    namespace
from random import randint
from util import gen_uuid

# This module parses and simplifies various elements of syntactic sugar

# Converts functions to transforms since they are really the same thing.
def func_to_trans(token):
    if(len(token.subs) > 0):
        for i, sub in enumerate(token.subs):
            token.subs[i] = func_to_trans(sub)
    
    for i, sub in enumerate(token.subs):
        if(sub.role == TokenRole.FUNCTION and sub.label[0] != '*'):
            if(not [i for i in sub.subs if i.__name__() == 'op_:']):
                sub.subs += [operator(':'), variable('return')]
    
    return token

# Used internally by untree_funcs to recursively build a list from a single instance of nested functions.
def untree_discover(token, current_list = [], uuid = ''):
    if(not uuid):
        uuid = gen_uuid()
    for i, sub in enumerate(token.subs):
        if(sub.role == TokenRole.FUNCTION and sub.label[0] != '*'):
            sub = untree_discover(sub, current_list, uuid)
            vname = '#temp' + uuid + str(len(current_list) // 2)
            token.subs[i] = variable(vname)
            current_list.append(statement('defineVar', [unknown('Auto'), string(vname)]))
            current_list.append(statement('assign', [variable(vname), sub]))
    return token

# Converts nested functions into a procedural list.
def untree_funcs(token):
    output = []
    for i, sub in enumerate(token.subs):
        if(sub.role == TokenRole.FUNCTION):
            temps = []
            sub = untree_discover(sub, temps)
            if(token.role == TokenRole.NAMESPACE): # We can just straight up insert the extra code.
                output += temps + [sub]
            else: # Mark that the parent function should deal with the code block, putting it in when possible.
                output = [namespace('temps', temps)] + output + [sub]
        elif(len(sub.subs) > 0):
            untreed = untree_funcs(sub)
            if(untreed.subs[0].__name__() == 'namespace_temps'): # It couldn't insert the temp code in its own code block, it needs to be in a higher level code block.
                if(token.role == TokenRole.NAMESPACE): # Temp code can go here.
                    output += untreed.subs[0].subs
                else: # Hand it up to be handled by the parent again.
                    output = [untreed.subs[0]] + output
                untreed.subs = untreed.subs[1:]
            output.append(untreed)
        else:
            output.append(sub)
    
    token.subs = output
    return token

def process(token):
    #print(token)
    token = func_to_trans(token)
    #token = untree_funcs(token)
    return token