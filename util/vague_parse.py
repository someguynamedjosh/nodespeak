#!/bin/env python3
import sys

class Enum:
    def __init__(self):
        self.name = ''
        self.items = []
    
    def add_item(self, item):
        self.items.append(item)
    
    def tree(self, indent=0):
        ind = '  ' * indent
        out = self.name + '(\n'
        for item in self.items:
            out += ind + '  '
            if type(item) is str:
                out += item + ',\n'
            else:
                out += item.tree(indent + 1) + ',\n'
        out += ind + ')'
        return out
    
    def get(self, index):
        return self.items[index]
    
    def keys(self):
        return list(range(len(self.items)))

class Struct:
    def __init__(self):
        self.name = ''
        self.items = {}
    
    def add_item(self, name, value):
        self.items[name] = value
    
    def tree(self, indent=0):
        ind = '  ' * indent
        out = self.name + ' {\n'
        for key, item in self.items.items():
            out += ind + '  ' + key + ': '
            if type(item) is str:
                out += item + ',\n'
            else:
                out += item.tree(indent + 1) + ',\n'
        out += ind + '}'
        return out
    
    def get(self, index):
        return self.items[index]
    
    def keys(self):
        return list(self.items.keys())

class Array:
    def __init__(self):
        self.items = []

    def add_item(self, item):
        self.items.append(item)
    
    def tree(self, indent=0):
        ind = '  ' * indent
        out = '[\n' 
        for item in self.items:
            out += ind + '  '
            if type(item) is str:
                out += item + ',\n'
            else:
                out += item.tree(indent + 1) + ',\n'
        out += ind + ']'
        return out
    
    def get(self, index):
        return self.items[index]
    
    def keys(self):
        return list(range(len(self.items)))

buffer = sys.stdin.read()
lines = [line for line in buffer.split('\n') if line not in ['Compiling source...']]

mode_stack = [None]
data_stack = []

def push_new(symbol, name):
    if symbol == '{':
        mode_stack.append('struct')
        to_push = Struct()
        to_push.name = name
        data_stack.append(to_push)
    elif symbol == '(':
        mode_stack.append('enum')
        to_push = Enum()
        to_push.name = name
        data_stack.append(to_push)
    elif symbol == '[':
        mode_stack.append('array')
        to_push = Array()
        data_stack.append(to_push)
    else:
        raise Exception('Unknown data symbol ' + symbol + ' (name ' + name + ')')

def pop():
    if (len(data_stack) > 1):
        data_stack.pop()
        mode_stack.pop()

for line in lines:
    line = line.strip()
    mode = mode_stack[-1]
    if (len(line) <= 1):
        continue
    if mode == None:
        symbol = line[-1]
        name = line[:-1].strip()
        push_new(symbol, name)
    elif mode == 'struct':
        if ':' in line:
            key = line.split(':')[0]
            symbol = line[-1]
            name = line.split(':')[1][:-1].strip()
            if (symbol == ','):
                data_stack[-1].add_item(key, name)
            else:
                push_new(symbol, name)
                data_stack[-2].add_item(key, data_stack[-1])
        else:
            pop()
    elif mode == 'array':
        if '],' in line:
            pop()
        else:
            symbol = line[-1]
            name = line[:-1].strip()
            if symbol == ',':
                data_stack[-1].add_item(name)
            else:
                push_new(symbol, name)
                data_stack[-2].add_item(data_stack[-1])
    elif mode == 'enum':
        if '),' in line:
            pop()
        else:
            symbol = line[-1]
            name = line[:-1].strip()
            if symbol == ',':
                data_stack[-1].add_item(name)
            else:
                push_new(symbol, name)
                data_stack[-2].add_item(data_stack[-1])

root = data_stack[0]
        
class Variable:
    def __init__(self, source):
        self.defining_scope = -1
        self.name = 'UNNAMED'
        self.type_name = ''
        self.matching_scope = None
        type_declaration = source.get('data_type')
        if type(type_declaration) == type(''):
            self.type_name = type_declaration
            if self.type_name == 'Function_':
                self.matching_scope = int(source.get('initial_value').get(0).get('body').get(0))
        else:
            self.type_name = 'todo...'
    
    def describe(self):
        global scopes
        scope_name = 'nowhere'
        if self.defining_scope != -1:
            scope_name = 'in ' + scopes[self.defining_scope].get_name()
        return self.name + ' (' + self.type_name + ' defined ' + scope_name + ')' + self.extra()
    
    def extra(self):
        return ''
    
    def finalize(self):
        global scopes
        if self.matching_scope is not None:
            scopes[self.matching_scope].name = self.name

variables = []
for variable_source in root.get('variables').items:
    variable = Variable(variable_source)
    variables.append(variable)

class Scope:
    def __init__(self):
        self.name = 'unnamed'
        self.parent = None
        self.body = []
    
    def get_name(self):
        global scopes
        if self.parent is None:
            return self.name
        else:
            return scopes[self.parent].get_name() + '.' + self.name
        
    def describe(self):
        out = 'SCOPE for ' + self.get_name() + '\n'
        out += ' body:\n'
        for call in self.body:
            out += '  ' + call.describe().replace('\n', '\n  ') + '\n'
        return out

class FuncCall:
    def __init__(self, function, inputs, outputs):
        self.function = function
        self.inputs = inputs
        self.outputs = outputs
    
    def describe(self):
        global variables
        out = 'call ' + variables[self.function].describe() + '\n'
        for index, value in enumerate(self.inputs):
            out += '  i' + str(index + 1) + ': ' + variables[value].describe() + '\n'
        for index, value in enumerate(self.outputs):
            out += '  o' + str(index + 1) + ': ' + variables[value].describe() + '\n'
        return out[:-1] # Strip trailing whitespace.


scopes = []
scope_sources = root.get('scopes').items
current_index = -1
for scope_source in scope_sources:
    current_index += 1
    scope = Scope()
    if current_index == 0:
        scope.name = 'root'
    parent_source = scope_source.get('parent')
    if parent_source != 'None':
        scope.parent = int(parent_source.get(0).get(0))
    scopes.append(scope)
    if scope_source.get('body') != '[]':
        for call in scope_source.get('body').items:
            function = int(call.get('function').get(0))
            inputs, outputs = [], []
            if call.get('inputs') != '[]':
                for iinput in call.get('inputs').items:
                    # TODO: Properly handle var access objects.
                    inputs.append(int(iinput.get('base').get(0)))
            if call.get('outputs') != '[]':
                for output in call.get('outputs').items:
                    # TODO: Properly handle var access objects.
                    outputs.append(int(output.get('base').get(0)))
            scope.body.append(FuncCall(function, inputs, outputs))
    if type(scope_source.get('symbols')) is not str:
        for symbol in scope_source.get('symbols').keys():
            real_name = symbol[1:-1] # Strip quotations.
            variable_id = scope_source.get('symbols').get(symbol).get(0)
            index = int(variable_id)
            variables[index].name = real_name
            variables[index].defining_scope = current_index
    if (scope_source.get('intermediates') == '[]'):
        continue
    for iindex, intermediate in enumerate(scope_source.get('intermediates').items):
        variable_id = intermediate.get(0)
        index = int(variable_id)
        variables[index].name = '!intermediate_' + str(iindex)
        variables[index].defining_scope = current_index

for variable in variables:
    variable.finalize()
for scope in scopes:
    print(scope.describe())