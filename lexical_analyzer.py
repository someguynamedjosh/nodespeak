from token import *

def process(text):
    operators = ['+', '-', '*', '/', '=', '+=', '-=', '*=', '/=', '%', '%=', 
               '==', '!=', '<', '>', '<=', '>=', '&', '|', '@', '!', 
               '==~', '!=~', '<~', '>~', '<=~', '>=~', '&~', '|~', '@~', '!~',
               ',', '.', ';', ':']
    symbols = operators + ['(', ')', '0x', '0b', '[', ']', '{', '}']
    symbols = sorted(symbols, key=lambda e: -len(e))
    whitespace = [' ', '\t', '\n']
    buf = ''
    stack = [namespace('root')]
    pop_triggers = [None]
    
    state = 0
    def append(lexeme):
        nonlocal state
        ta = None
        if(state == 0): # Just names and stuff
            if(lexeme in operators):
                if((lexeme == '.') and (len(stack[-1].subs) > 0) and (stack[-1].subs[-1].role == TokenRole.NUMBER)):
                    state = 3 # Append fractional portion to decimal number.
                else:
                    ta = operator(lexeme)
            elif(lexeme[0] == '0'):
                if(lexeme == '0'):
                    ta = number(0)
                elif(lexeme == '0b'):
                    state = 1 # Binary number
                    return
                elif(lexeme == '0x'):
                    state = 2 # Hexadecimal number
                    return
                else: # Octal number
                    ta = number(float(int(lexeme))) # Python automatically interprets numbers starting with 0 as octal.
            elif(lexeme[0] in '123456789'): # Just a normal number.
                ta = number(float(lexeme))
            else:
                ta = unknown(lexeme)
        elif(state == 1): # Binary number
            ta = number(float(int(lexeme, 2)))
            state = 0
        elif(state == 2): # Hexadecimal number
            ta = number(float(int(lexeme, 16)))
        elif(state == 3): # Append post-decimal portion to number
            pd = float(int(lexeme, 10))
            pd /= 10 ** len(lexeme) # 1 digit, divide by 10, 2, divide by 100, etc.
            stack[-1].subs[-1].numeric += pd
            stack[-1].subs[-1].label = str(stack[-1].subs[-1].numeric)
            state = 0
        if(ta):
            stack[-1].subs.append(ta)
    
    i = 0
    while i < len(text):
        for s in symbols:
            if(text[i:i+len(s)] == s): # Split whenever a symbol is found
                if(buf):
                    append(buf)
                    buf = ''
                if(s in '([{'): # If the symbol is an open brace, create a new group.
                    stack.append(group(s))
                    pop_triggers.append(')]}'['([{'.find(s)])
                elif(s == pop_triggers[-1]): # If the symbol is the correct closing brace, close the previous group.
                    stack[-2].subs.append(stack[-1])
                    stack = stack[:-1]
                    pop_triggers = pop_triggers[:-1]
                else: # Otherwise, just append the symbol.
                    append(s)
                    i += len(s) - 1
                break
        else: # If the text does not contain a symbol...
            c = text[i]
            if c in whitespace: # If the current character is a space or tab or newline, split off the current symbol.
                if(buf):
                    append(buf)
                    buf = ''
                    # Don't append the whitespace as a symbol because it is whitespace.
            else:
                buf += c
        i += 1
    if(buf):
        append(buf)
    
    return stack[-1]