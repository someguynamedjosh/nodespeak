class DeterminateFiniteAutomaton:
    def __init__(self):
        self.transitions = {}
        self.state_labels = []
        self.state_counter = -1
    
    def add_state(self, label=None):
        self.state_counter += 1
        self.transitions[self.state_counter] = {}
        self.state_labels.append(label)
        return self.state_counter
    
    def set_state_label(self, state, label):
        self.state_labels[state] = label
        
    def get_state_label(self, state):
        return self.state_labels[state]
    
    def set_transition(self, start_state, trigger, end_state):
        assert trigger != '' and trigger != '\x00' # empty string transitions not allowed in DFAs
        # Add extra states if they are not already part of the graph.
        while(self.state_counter < start_state):
            self.add_state()
        while(self.state_counter < end_state):
            self.add_state()
        self.transitions[start_state][trigger] = end_state
    
    def __str__(self):
        out = '===== BEGIN DFA DESCRIPTION =====\n'
        out += str(self.state_counter + 1) + ' state(s)\n'
        for i in range(self.state_counter + 1):
            if(self.state_labels[i] is not None):
                out += 'State ' + str(i) + ' has label ' + str(self.state_labels[i]) + '\n'
            for trigger in self.transitions[i].keys():
                out += str(i) + ' -> "' + trigger + '" -> ' + str(self.transitions[i][trigger]) + '\n'
        out += '====== END DFA DESCRIPTION ======'
        return out

class NondeterminateFiniteAutomaton:
    def __init__(self):
        self.transitions = {}
        self.state_labels = []
        self.state_counter = -1
    
    def add_state(self, label=None):
        self.state_counter += 1
        self.transitions[self.state_counter] = {}
        self.state_labels.append(label)
        return self.state_counter
    
    def set_state_label(self, state, label):
        self.state_labels[state] = label
        
    def get_state_label(self, state):
        return self.state_labels[state]
    
    def add_transition(self, start_state, trigger, end_state):
        # Add extra states if they are not already part of the graph.
        while(self.state_counter < start_state):
            self.add_state()
        while(self.state_counter < end_state):
            self.add_state()
        if(trigger not in self.transitions[start_state].keys()):
            self.transitions[start_state][trigger] = []
        self.transitions[start_state][trigger].append(end_state)
    
    def __str__(self):
        out = '===== BEGIN NFA DESCRIPTION =====\n'
        out += str(self.state_counter + 1) + ' state(s)\n'
        for i in range(self.state_counter + 1):
            if(self.state_labels[i] is not None):
                out += 'State ' + str(i) + ' has label ' + str(self.state_labels[i]) + '\n'
            for trigger in self.transitions[i].keys():
                out += str(i) + ' -> "' + trigger + '" -> ' + str(self.transitions[i][trigger]) + '\n'
        out += '====== END NFA DESCRIPTION ======'
        return out
    
    def convert_to_dfa(self):
        out = DeterminateFiniteAutomaton()
        multi_state_map = {} # Keeps track of the states in the DFA that represent combinations of states in the NFA.
        for i in range(self.state_counter + 1):
            multi_state_map[(i,)] = out.add_state() # For when looking up a 'combo state' of one state.
        def get_dfa_state(superposition): # Given a superposition of multiple possible states, get (or create) the corresponding DFA state.
            if(superposition not in multi_state_map.keys()):
                # The multi_state_map dict contains which DFA states correspond to each superposition.
                multi_state_map[superposition] = out.add_state()
                convert_transition(superposition) # Find what will happen with this superposition.
            return multi_state_map[superposition]
        def convert_transition(from_states): # Finds all of the possible outcomes of starting from any state from a list of states.
            transitions = {} # A dictionary filled with all the possible outcomes of any given trigger when starting from any of the given start states.
            labels = [self.get_state_label(i) for i in from_states]
            from_dfa_state = get_dfa_state(from_states)
            def add_transitions(from_states): # Sub function that is used to recursively search through possible transitions when a blank transition is encountered.
                nonlocal transitions, labels
                for from_state in from_states:
                    for trigger in self.transitions[from_state].keys():
                        if(trigger == '' or trigger == '\x00'): # \x00 will be the representation of an empty string transition in the c++ implementation.
                            to_states = self.transitions[from_state][trigger]
                            labels += [self.get_state_label(i) for i in to_states]
                            # Directly add the transitions from what will be transitioned to, to flatten and remove empty string transitions.
                            add_transitions(to_states)
                        else:
                            if(trigger not in transitions.keys()):
                                transitions[trigger] = []
                            transitions[trigger] += self.transitions[from_state][trigger] # Add the destination of this transition as a possible outcome for the given superposition of inputs.
            add_transitions(from_states) # Start the recursion off with the root input.
            for trigger in transitions.keys(): # Right now, the dict pairs triggers with what outcomes could possibly happen. This loop converts those to DFA states representing the superposition of those possibilities.
                out.set_transition(from_dfa_state, trigger, get_dfa_state(tuple(set(transitions[trigger]))))
            labels = set([i for i in labels if i is not None])
            if(len(labels) > 0):
                out.set_state_label(get_dfa_state(from_states), max(labels))
        for i in range(self.state_counter + 1):
            convert_transition((i,))
        return out

if __name__ == '__main__':
    nfa = NondeterminateFiniteAutomaton()
    a, b, c, d, e = [nfa.add_state() for i in range(5)]
    nfa.add_transition(a, '1', b)
    nfa.add_transition(a, '1', c)
    nfa.add_transition(b, '2', c)
    nfa.add_transition(c, '3', a)
    nfa.add_transition(d, '', e)
    nfa.add_transition(e, '5', a)
    
    print(nfa)
    print(nfa.convert_to_dfa())
