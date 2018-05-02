class Datatype:
    def __init__(self, name, size):
        self.name, self.size = name, size
    
    def __str__(self):
        return (self.name or '') + ' (' + str(self.size) + ' bytes)'

class ArrayDatatype(Datatype):
    def __init__(self, element_type, length):
        assert element_type is not None
        super().__init__(element_type.name + '[' + str(length) + ']', element_type.size * length)

class ObjectDatatype(Datatype):
    def __init__(self, members):
        self.members = members
        total_size = sum([m.datatype.size for m in members])
        super().__init__(None, total_size)
    
    def __str__(self):
        out = super().__str__() + ' {\n'
        for member in self.members:
            out += '\t' + str(member).replace('\n', '\n\t')
        out += '\n}'
        return out

class Variable:
    def __init__(self, datatype, name):
        self.datatype, self.name = datatype, name
    
    def __str__(self):
        return 'variable_' + self.name + ': {\n\tdatatype: ' + str(self.datatype) + '\n}'
        
class DefaultableVariable:
    def __init__(self, datatype, name, default_value = None):
        self.datatype, self.name, self.default_value = datatype, name, default_value   
    
    def __str__(self):
        return 'variable_' + self.name + ': {\n\tdatatype: ' + str(self.datatype) + '\n\tdefault value: ' + str(self.default_value) + '\n}'     
        
class Function:
    def __init__(self, name, inputs, outputs, code_scope):
        self.name, self.inputs, self.outputs, self.scope = name, inputs, outputs, code_scope
        for v in inputs + outputs:
            self.scope.add_variable(v)
    
    def __str__(self):
        output = ''
        output += 'function_' + self.name + ': {\n'
        output += '\tinputs: {\n\t\t'
        for v in self.inputs:
            output += str(v).replace('\n', '\n\t\t') + ', '
        output = output[:-2]
        output += '\n\t}, outputs: {\n\t\t'
        for v in self.outputs:
            output += str(v).replace('\n', '\n\t\t') + ', '
        output = output[:-2]
        output += '\n\t}, ' + str(self.scope).replace('\n', '\n\t') + '\n}'
        return output

class Literal:
    def __init__(self, datatype, value):
        self.datatype, self.value = datatype, value        

class Scope:
    def __init__(self, parent = None):
        self.datatypes = []
        self.variables = []
        self.functions = []
        self.statements = []
        self.parent = parent
        
    def find(self, list, name, pfunc=None):
        for item in list:
            if(item.name == name):
                return item
        if(pfunc):
            return pfunc(name)
        else:
            return None
    
    def find_datatype(self, name):
        return self.find(self.datatypes, name, self.parent and self.parent.find_datatype)
    
    def find_variable(self, name):
        return self.find(self.variables, name, self.parent and self.parent.find_variable)
    
    def find_function(self, name):
        return self.find(self.functions, name, self.parent and self.parent.find_function)
    
    def add_datatype(self, datatype):
        self.datatypes.append(datatype)
    
    def add_variable(self, variable):
        self.variables.append(variable)
    
    def add_function(self, function):
        self.functions.append(function)
    
    def add_statement(self, statement):
        self.statements.append(statement)
    
    def __str__(self):
        output = ''
        output += 'scope: {\n\tdatatypes: {\n\t\t'
        for d in self.datatypes:
            output += str(d).replace('\n', '\n\t\t') + ', '
        output = output[:-2]
        
        output += '\n\t}, variables: {\n\t\t'
        for d in self.variables:
            output += str(d).replace('\n', '\n\t\t') + ', '
        output = output[:-2]
        
        output += '\n\t}, functions: {\n\t\t'
        for d in self.functions:
            output += str(d).replace('\n', '\n\t\t') + ', '
        output = output[:-2]
        
        output += '\n\t}, statements: {\n\t\t'
        for d in self.statements:
            output += str(d).replace('\n', '\n\t\t') + ', '
        output = output[:-2]
        
        output += '\n\t}\n}'
        return output
    
    