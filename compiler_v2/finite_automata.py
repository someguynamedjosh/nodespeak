class DeterminateFiniteAutomaton:
    def __init__(self):
        self.transitions = {}
        self.state_labels = {}
        self.state_counter = 0
        self.start_state = None
    
    def add_state(self):
        self.transitions[self.state_counter] = {}
        self.state_labels[self.state_counter] = set()
        self.state_counter += 1
        return self.state_counter - 1
    
    def get_states(self):
        return range(self.state_counter)
    
    def add_state_label(self, state, label):
        self.state_labels[state].add(label)
        
    def get_state_labels(self, state):
        return self.state_labels[state]
    
    def set_transition(self, start_state, trigger, end_state):
        assert trigger != '\x00' # empty string transitions not allowed in DFAs
        # Add extra states if they are not already part of the graph.
        while(self.state_counter <= start_state):
            self.add_state()
        while(self.state_counter <= end_state):
            self.add_state()
        if(trigger not in self.transitions[start_state].keys()):
            self.transitions[start_state][trigger] = set()
        self.transitions[start_state][trigger] = end_state
    
    def get_transitions_from(self, start_state):
        return self.transitions[start_state].items()
    
    def test(self, input):
        state = self.start_state
        stacktrace = [(state,)]
        for char in input:
            try:
                state = self.transitions[state][char]
                stacktrace.append((char, state))
            except:
                print('Invalid transition from state', state, 'on trigger', char)
                print('Stacktrace:', stacktrace)
                return
        print('DFA finished without errors.')
        print('Stacktrace:', stacktrace)
        return self.get_state_labels(state)
    
    def run(self, input):
        state = self.start_state
        for char in input:
            try:
                state = self.transitions[state][char]
            except:
                raise Exception('The input ' + input + ' is invalid for this DFA.')
        return self.get_state_labels(state)
    
    def reset_run(self): # Get ready for calling step() repeatedly.
        self.state = self.start_state
    
    def step(self, char):
        try:
            self.state = self.transitions[self.state][char]
        except:
            raise Exception('The character ' + char + ' is not a valid transition from state ' + str(self.state) + '.')
        return self.get_state_labels(self.state)
    
    def get_current_states(self):
        return self.get_state_labels(self.state) 
    
    def __str__(self):
        out = '===== BEGIN DFA DESCRIPTION =====\n'
        out += str(self.state_counter) + ' state(s)\n'
        for i in range(self.state_counter):
            if(len(self.get_state_labels(i))):
                out += 'State ' + str(i) + ' has label(s) ' + ', '.join([str(i) for i in self.get_state_labels(i)]) + '\n'
            if(i == self.start_state):
                out += 'State ' + str(i) + ' is the starting point.\n'
            for trigger in self.transitions[i].keys():
                out += str(i) + ' -> ' + repr(trigger) + ' -> ' + str(self.transitions[i][trigger]) + '\n'
        out += '====== END DFA DESCRIPTION ======'
        return out    

class NondeterminateFiniteAutomaton:
    def __init__(self):
        self.transitions = {}
        self.state_labels = {}
        self.state_counter = 0
        self.start_state = None
    
    def add_state(self):
        self.transitions[self.state_counter] = {}
        self.state_labels[self.state_counter] = set()
        self.state_counter += 1
        return self.state_counter - 1
    
    def add_states(self, source_nfa):
        offset = self.state_counter
        for state, transition_table in source_nfa.transitions.items():
            self.transitions[state + offset] = dict(transition_table)
            for trigger, to_states in self.transitions[state + offset].items():
                self.transitions[state + offset][trigger] = set([i + offset for i in to_states])
            self.state_labels[state + offset] = set(source_nfa.state_labels[state])
        self.state_counter += source_nfa.state_counter
        return offset
    
    def get_states(self):
        return range(self.state_counter)
    
    def add_state_label(self, state, label):
        self.state_labels[state].add(label)
        
    def get_state_labels(self, state):
        return self.state_labels[state]
    
    def add_transition(self, start_state, trigger, end_state):
        # Add extra states if they are not already part of the graph.
        while(self.state_counter <= start_state):
            self.add_state()
        while(self.state_counter <= end_state):
            self.add_state()
        if(trigger not in self.transitions[start_state].keys()):
            self.transitions[start_state][trigger] = set()
        self.transitions[start_state][trigger].add(end_state)
    
    def get_transitions_from(self, start_state):
        return self.transitions[start_state].items()
    
    def __str__(self):
        out = '===== BEGIN NFA DESCRIPTION =====\n'
        out += str(self.state_counter) + ' state(s)\n'
        for i in range(self.state_counter):
            if(len(self.get_state_labels(i))):
                out += 'State ' + str(i) + ' has label(s) ' + ', '.join([str(i) for i in self.get_state_labels(i)]) + '\n'
            if(i == self.start_state):
                out += 'State ' + str(i) + ' is the starting point.\n'
            for trigger in self.transitions[i].keys():
                out += str(i) + ' -> ' + repr(trigger) + ' -> ' + str(self.transitions[i][trigger]) + '\n'
        out += '====== END NFA DESCRIPTION ======'
        return out
    
    def convert_to_dfa(self):
        dfa = DeterminateFiniteAutomaton()
        epsilon_closures = {} # An epsilon closure is the list of states, including the starting state, that can be reached from the start state, using only zero or more epsilon transitions.
        def traverse_epsilon_transitions(state, exclude=set()):
            out = exclude.union({state})
            if('\x00' in self.transitions[state].keys()):
                for to_state in self.transitions[state]['\x00']:
                    if(to_state not in out):
                        out.add(to_state)
                        for sub_state in traverse_epsilon_transitions(to_state, out):
                            out.add(sub_state)
            return out
        for state in range(self.state_counter):
            epsilon_closures[state] = traverse_epsilon_transitions(state)
            
        # Compute a DFA structure using combinations of states.
        table = {} # Table of states and their transitions.
        states = [epsilon_closures[self.start_state]]
        index = 0
        while index < len(states):
            from_states = frozenset(states[index])
            table[from_states] = {}
            for from_state in from_states:
                for trigger, sub_states in [i for i in self.transitions[from_state].items() if i[0] != '\x00']:
                    to_states = set()
                    for sub_state in sub_states:
                        to_states = to_states.union(epsilon_closures[sub_state])
                    if(trigger not in table[from_states].keys()):
                        table[from_states][trigger] = set()
                    table[from_states][trigger] = table[from_states][trigger].union(to_states)
            for trigger, to_states in table[from_states].items():
                if(to_states not in states):
                    states.append(to_states)
            index += 1
        dfa_map = {}
        for from_states in table.keys():
            dfa_map[from_states] = dfa.add_state()
            for from_state in from_states:
                for label in self.get_state_labels(from_state):
                    dfa.add_state_label(dfa_map[from_states], label)
        
        # Transfer labels and entry point from the NFA to the DFA
        dfa.start_state = dfa_map[frozenset(epsilon_closures[self.start_state])]
        for from_states, transitions in table.items():
            for trigger, to_states in transitions.items():
                dfa.set_transition(dfa_map[from_states], trigger, dfa_map[frozenset(to_states)])
        return dfa

if __name__ == '__main__':
    nfa = NondeterminateFiniteAutomaton()
    a, b, c, d, e = [nfa.add_state() for i in range(5)]
    nfa.add_transition(a, '1', b)
    nfa.add_transition(a, '1', c)
    nfa.add_transition(b, '2', c)
    nfa.add_transition(c, '3', a)
    nfa.add_transition(d, '\x00', e)
    nfa.add_transition(e, '5', a)
    
    print(nfa)
    print(nfa.convert_to_dfa())
