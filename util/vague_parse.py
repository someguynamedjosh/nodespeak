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
lines = buffer.split('\n')

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
        
class Entity:
    def __init__(self):
        self.defining_scope = -1
        self.name = 'UNNAMED'
        self.type_name = ''
    
    def describe(self):
        global scopes
        scope_name = 'nowhere'
        if self.defining_scope != -1:
            scope_name = 'in ' + scopes[self.defining_scope].get_name()
        return self.name + ' (' + self.type_name + ' defined ' + scope_name + ')' + self.extra()
    
    def extra(self):
        return ''
    
    def finalize(self):
        pass

class VariableEntity(Entity):
    def __init__(self, data_type_id):
        Entity.__init__(self)
        self.data_type_id = int(data_type_id)
        self.data_type_name = 'UNDISCOVERED'
        self.type_name = 'Variable'
    
    def extra(self):
        global entities
        return ' {type: ' + entities[self.data_type_id].describe() + '}'

class FunctionEntity(Entity):
    def __init__(self):
        Entity.__init__(self)
        self.type_name = 'Function'
        self.body = -1
    
    def extra(self):
        global scopes
        name = '(nonexistant)'
        if self.body > -1:
            name = scopes[self.body].get_name()
        return ' {body: ' + name + '}'
    
    def finalize(self):
        global scopes
        if self.body > -1:
            scopes[self.body].name = self.name

class BuiltinFunction(Entity):
    def __init__(self):
        Entity.__init__(self)
        self.type_name = 'BuiltinFunction'
        self.body = -1
    
    def extra(self):
        global scopes
        name = '(nonexistant)'
        if self.body > -1:
            name = scopes[self.body].get_name()
        return ' {body: ' + name + '}'
    
    def finalize(self):
        global scopes
        if self.body > -1:
            scopes[self.body].name = self.name

class IntLiteral(Entity):
    def __init__(self):
        Entity.__init__(self)
        self.type_name = 'IntLiteral'

class FloatLiteral(Entity):
    def __init__(self):
        Entity.__init__(self)
        self.type_name = 'FloatLiteral'

class DataTypeEntity(Entity):
    def __init__(self, contained_type):
        Entity.__init__(self)
        self.contained_type = contained_type
        self.type_name = 'DataType'
    
    def extra(self):
        return ' {defined as ' + self.contained_type + '}'

entities = []
for entity_source in root.get('entities').items:
    name = entity_source.name
    entity = None
    if name == 'Variable':
        struct = entity_source.get(0)
        entity = VariableEntity(struct.get('data_type').get('raw_id'))
    elif name == 'Function':
        entity = FunctionEntity()
        struct = entity_source.get(0)
        entity.body = int(struct.get('body').get('raw_id'))
    elif name == 'BuiltinFunction':
        entity = BuiltinFunction()
        struct = entity_source.get(0).get('base')
        entity.body = int(struct.get('body').get('raw_id'))
    elif name == 'IntLiteral':
        entity = IntLiteral()
    elif name == 'FloatLiteral':
        entity = FloatLiteral()
    elif name == 'DataType':
        entity = DataTypeEntity(entity_source.items[0])
    entities.append(entity)

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
        out = 'SCOPE ' + self.get_name() + '\n'
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
        global entities
        out = 'call ' + entities[self.function].describe() + '\n'
        for index, value in enumerate(self.inputs):
            out += '  i' + str(index + 1) + ': ' + entities[value].describe() + '\n'
        for index, value in enumerate(self.outputs):
            out += '  o' + str(index + 1) + ': ' + entities[value].describe() + '\n'
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
        scope.parent = int(parent_source.get(0).get('raw_id'))
    scopes.append(scope)
    if scope_source.get('body') != '[]':
        for call in scope_source.get('body').items:
            function = int(call.get('function').get('raw_id'))
            inputs, outputs = [], []
            if call.get('inputs') != '[]':
                for iinput in call.get('inputs').items:
                    # TODO: Properly handle var access objects.
                    inputs.append(int(iinput.get('base').get('raw_id')))
            if call.get('outputs') != '[]':
                for output in call.get('outputs').items:
                    # TODO: Properly handle var access objects.
                    outputs.append(int(output.get('base').get('raw_id')))
            scope.body.append(FuncCall(function, inputs, outputs))
    if type(scope_source.get('symbols')) is not str:
        for symbol in scope_source.get('symbols').keys():
            real_name = symbol[1:-1] # Strip quotations.
            entity_id = scope_source.get('symbols').get(symbol).get('raw_id')
            index = int(entity_id)
            entities[index].name = real_name
            entities[index].defining_scope = current_index
    if (scope_source.get('intermediates') == '[]'):
        continue
    for iindex, intermediate in enumerate(scope_source.get('intermediates').items):
        entity_id = intermediate.get('raw_id')
        index = int(entity_id)
        entities[index].name = '!intermediate_' + str(iindex)
        entities[index].defining_scope = current_index

for entity in entities:
    entity.finalize()
for scope in scopes:
    print(scope.describe())