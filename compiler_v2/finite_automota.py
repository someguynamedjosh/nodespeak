class DeterminateNode:
    def __init__(self, transitions = {}):
        self.transitions = transitions
    
    def __getitem__(self, key):
        if(key in self.transitions):
            return self.transitions[key]
        else:
            return None
    
    def __setitem__(self, key, value):
        assert type(key) is str
        assert len(key) == 1
        if value is None:
            if(key in self.transitions):
                del self.transitions[key]
        else:
            assert type(value) is DeterminateNode
            self.transitions[key] = value
    
    def __delitem__(self, key):
        assert type(key) is str
        assert len(key) == 1
        if(key in self.transitions):
            del self.transitions[key]        
        
DNode = DeterminateNode

class NondeterminateNode:
    uuid_counter = 0
    def __init__(self, transitions = {}):
        self.transitions = transitions
        self.uuid = NondeterminateNode.uuid_counter
        NondeterminateNode.uuid_counter += 1
    
    def __getitem__(self, key):
        if(key not in self.transitions):
            self.transitions[key] = []
        return self.transitions[key]
    
    def __setitem__(self, key, value):
        assert type(key) is str
        assert len(key) <= 1
        if value is None:
            if(key in self.transitions):
                del self.transitions[key]
        else:
            if(isinstance(value, NondeterminateNode)):
                self.transitions[key] = [value]
            elif(type(value) is list and min([isinstance(i, NondeterminateNode) for i in value])):
                self.transitions[key] = value
            else:
                assert False
    
    def __delitem__(self, key):
        assert type(key) is str
        assert len(key) <= 1
        if(key in self.transitions):
            del self.transitions[key]
    
    def __radd__(self, addend):
        if(type(addend) is list):
            return addend + [self]
        elif(isinstance(addend, NondeterminateNode)):
            return [self, addend]
    
    def __str__(self):
        return 'Node #' + str(self.uuid) + ' (' + str(sum([len(i) for i in self.transitions.items()])) + ' transitions)'
    
    def __repr__(self):
        return str(self)

NNode = NondeterminateNode

if __name__ == '__main__':
    a, b, c = NNode(), NNode(), NNode()
    a['1'].append(b)
    a['1'] += c
    
    print(a['1'])

