class DeterminateFiniteAutomota:
    def __init__(self):
        self.transitions = {}
        self.node_counter = -1
    
    def add_node(self):
        self.node_counter += 1
        self.transitions[self.node_counter] = {}
        return self.node_counter
    
    def set_transition(self, start_state, trigger, end_state):
        assert trigger != '' and trigger != '\x00' # empty string transitions not allowed in DFAs
        # Add extra nodes if they are not already part of the graph.
        while(self.node_counter < start_state):
            self.add_node()
        while(self.node_counter < end_state):
            self.add_node()
        self.transitions[start_state][trigger] = end_state
    
    def __str__(self):
        out = '===== BEGIN DFA DESCRIPTION =====\n'
        out += str(self.node_counter + 1) + ' node(s)\n'
        for i in range(self.node_counter + 1):
            for trigger in self.transitions[i].keys():
                out += str(i) + ' -> "' + trigger + '" -> ' + str(self.transitions[i][trigger]) + '\n'
        out += '====== END DFA DESCRIPTION ======'
        return out

class NondeterminateFiniteAutomota:
    def __init__(self):
        self.transitions = {}
        self.node_counter = -1
    
    def add_node(self):
        self.node_counter += 1
        self.transitions[self.node_counter] = {}
        return self.node_counter
    
    def add_transition(self, start_state, trigger, end_state):
        # Add extra nodes if they are not already part of the graph.
        while(self.node_counter < start_state):
            self.add_node()
        while(self.node_counter < end_state):
            self.add_node()
        if(trigger not in self.transitions[start_state].keys()):
            self.transitions[start_state][trigger] = []
        self.transitions[start_state][trigger].append(end_state)
    
    def __str__(self):
        out = '===== BEGIN NFA DESCRIPTION =====\n'
        out += str(self.node_counter + 1) + ' node(s)\n'
        for i in range(self.node_counter + 1):
            for trigger in self.transitions[i].keys():
                out += str(i) + ' -> "' + trigger + '" -> ' + str(self.transitions[i][trigger]) + '\n'
        out += '====== END NFA DESCRIPTION ======'
        return out
    
    def convert_to_dfa(self):
        pass

if __name__ == '__main__':
    nfa = NondeterminateFiniteAutomota()
    a, b, c = [nfa.add_node() for i in range(3)]
    nfa.add_transition(a, '1', b)
    nfa.add_transition(a, '1', c)
    nfa.add_transition(b, '2', c)
    nfa.add_transition(c, '3', a)
    
    print(nfa)

