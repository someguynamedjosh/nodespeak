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
        out = DeterminateFiniteAutomota()
        multi_state_map = {} # Keeps track of the states in the DFA that represent combinations of states in the NFA.
        for i in range(self.node_counter + 1):
            multi_state_map[(i,)] = out.add_node() # For when looking up a 'combo state' of one state.
        def get_dfa_state(superposition): # Given a superposition of multiple possible states, get (or create) the corresponding DFA state.
            if(superposition not in multi_state_map.keys()):
                # The multi_state_map dict contains which DFA states correspond to each superposition.
                multi_state_map[superposition] = out.add_node()
                convert_transition(superposition) # Find what will happen with this superposition.
            return multi_state_map[superposition]
        def convert_transition(from_states): # Finds all of the possible outcomes of starting from any state from a list of states.
            transitions = {} # A dictionary filled with all the possible outcomes of any given trigger when starting from any of the given start states.
            from_dfa_state = get_dfa_state(from_states)
            def add_transitions(from_states): # Sub function that is used to recursively search through possible transitions when a blank transition is encountered.
                nonlocal transitions
                for from_state in from_states:
                    for trigger in self.transitions[from_state].keys():
                        if(trigger == '' or trigger == '\x00'): # \x00 will be the representation of an empty string transition in the c++ implementation.
                            # Directly add the transitions from what will be transitioned to, to flatten and remove empty string transitions.
                            add_transitions(self.transitions[from_state][trigger])
                        else:
                            if(trigger not in transitions.keys()):
                                transitions[trigger] = []
                            transitions[trigger] += self.transitions[from_state][trigger] # Add the destination of this transition as a possible outcome for the given superposition of inputs.
            add_transitions(from_states) # Start the recursion off with the root input.
            for trigger in transitions.keys(): # Right now, the dict pairs triggers with what outcomes could possibly happen. This loop converts those to DFA states representing the superposition of those possibilities.
                out.set_transition(from_dfa_state, trigger, get_dfa_state(tuple(set(transitions[trigger]))))
        for i in range(self.node_counter + 1):
            convert_transition((i,))
        return out

if __name__ == '__main__':
    nfa = NondeterminateFiniteAutomota()
    a, b, c, d, e = [nfa.add_node() for i in range(5)]
    nfa.add_transition(a, '1', b)
    nfa.add_transition(a, '1', c)
    nfa.add_transition(b, '2', c)
    nfa.add_transition(c, '3', a)
    nfa.add_transition(d, '', e)
    nfa.add_transition(e, '5', a)
    
    print(nfa)
    print(nfa.convert_to_dfa())
