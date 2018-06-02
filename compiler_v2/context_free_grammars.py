from finite_automata import NondeterminateFiniteAutomaton

class AbstractSyntaxTree:
    def __init__(self, name):
        self.children = []
        self.name = str(name)
    
    def add_child(self, child):
        self.children.append(child)
    
    def __str__(self):
        if(len(self.children) == 0):
            return self.name
        output = self.name + ': \n'
        for child in self.children:
            output += '|   ' + str(child).replace('\n', '\n|   ') + '\n'
        return output[:-1] # Trim the last newline
    
    def __repr__(self):
        return str(self)

# Constants for CFGFA transition types.
TRANSITION_GO = 0 # Transition to another state (specified by extra data) without touching the stack.
TRANSITION_SHIFT = 1 # Transition to another state (specified by extra data) and push the next symbol from the input onto the stack.
TRANSITION_REDUCE = 2 # Replace 1 or more tokens on the stack with a nonterminal, as specified by a production (which is specified by extra data.)
TRANSITION_ACCEPT = 3 # The syntax tree was parsed successfully.

class CFGFiniteAutomaton: # Like a DFA, but has a stack for generating ASTs as well as more complex transitions to utilize the stack.
    def __init__(self, cfg):
        self.transitions = {}
        self.state_labels = {}
        self.state_counter = 0
        self.start_state = None
        self.cfg = cfg
    
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
    
    def set_transition(self, start_state, trigger, transition_type, extra_data):
        assert trigger != '\x00' # empty string transitions not allowed in DFAs
        # Add extra states if they are not already part of the graph.
        while(self.state_counter <= start_state):
            self.add_state()
        if(transition_type in [TRANSITION_GO, TRANSITION_SHIFT]): # These use the extra data as an end state.
            while(self.state_counter <= extra_data):
                self.add_state()
        if(trigger not in self.transitions[start_state].keys()):
            self.transitions[start_state][trigger] = set()
        self.transitions[start_state][trigger] = (transition_type, extra_data)
    
    def get_transitions_from(self, start_state):
        return self.transitions[start_state].items()
    
    def test(self, input, input_transform=lambda e: e):
        stack = [self.start_state]
        input = list(input)
        ast_stack = []
        index = 0
        while True:
            try:
                transition = self.transitions[stack[-1]][input_transform(input[index])]
                if(transition[0] == TRANSITION_GO):
                    stack.append(transition[1])
                elif(transition[0] == TRANSITION_REDUCE):
                    production = self.cfg.productions[transition[1]]
                    size = len(production.pattern)
                    ast = AbstractSyntaxTree(production.nonterminal) # Create a new root.
                    if(size > 0): # Add some terminals / nonterminals as children, but if there are 0, then it will try to add everything as children, an undesirable effect.
                        for t in ast_stack[-size:]:
                            ast.add_child(t) # Add leaves to the root.
                        ast_stack = ast_stack[:-size] # Remove ASTs and DFT states for leaves that were added.
                        stack = stack[:-size]
                    ast_stack.append(ast) # Add it in to the stack, so that it will eventually either be the root state of the entire tree or will become a leaf of another larger root.
                    if(self.transitions[stack[-1]][production.nonterminal][0] == TRANSITION_ACCEPT): # If the new nonterminal was put on the stack and would cause an accept, then do an accept.
                        return ast_stack[0]
                    else: # Put the correct state onto the stack.
                        stack.append(self.transitions[stack[-1]][production.nonterminal][1]) # Push DFA state onto the stack as well.
                        # Any transition on a nonterminal cannot be reducing. Reduction transitions occur when one of the TERMINALS from FOLLOW(nonterminal) is encountered.
                        # Thus, no reduction transitions will occur on NONTERMINALS. This means no recursive headache.
                elif(transition[0] == TRANSITION_SHIFT):
                    ast_stack.append(AbstractSyntaxTree(input[index]))
                    index += 1
                    stack.append(transition[1])
                elif(transition[0] == TRANSITION_ACCEPT):
                    return ast_stack[0]
            except:
                raise Exception('No valid transition from state ' + str(stack[-1]) + ' on trigger ' + str(input_transform(input[index])))
            print(stack, ast_stack, input[index:])
    
    def __str__(self):
        out = '===== BEGIN CFGFA DESCRIPTION =====\n'
        out += str(self.state_counter) + ' state(s)\n'
        for i in range(self.state_counter):
            if(len(self.get_state_labels(i))):
                out += 'State ' + str(i) + ' has label(s) ' + ', '.join([str(i) for i in self.get_state_labels(i)]) + '\n'
            if(i == self.start_state):
                out += 'State ' + str(i) + ' is the starting point.\n'
            human_readable = {
                TRANSITION_GO: 'go to state ',
                TRANSITION_REDUCE: 'reduce with prod. #',
                TRANSITION_SHIFT: 'add to stack and go to state ',
                TRANSITION_ACCEPT: 'accept and finalize syntax tree '
            }
            for trigger in self.transitions[i].keys():
                transition = self.transitions[i][trigger]
                out += str(i) + ' -> ' + repr(trigger) + ': ' + human_readable[transition[0]] + str(transition[1]) + '\n'
        out += '====== END CFGFA DESCRIPTION ======'
        return out

ASSOC_LEFT = 0
ASSOC_RIGHT = 1
    
class ContextFreeGrammar:
    def __init__(self):
        self.productions = []
        self.operators = {}
    
    def create_production(self, nonterminal, *pattern, **kwargs):
        self.productions.append(Production(self, nonterminal, *pattern, **kwargs))
        return self.productions[-1]
    
    def create_binary_operator_production(self, nonterminal, operator_pattern, **kwargs):
        assert operator_pattern not in self.operators.keys()
        self.operators[operator_pattern] = len(self.productions) # Pointer to the production for this operator.
        self.create_production(nonterminal, nonterminal, operator_pattern, nonterminal, is_binary_operator=True, **kwargs)
    
    def follow(self, nonterminal):
        if(type(nonterminal) is not tuple):
            nonterminal = (nonterminal,)
        old_length = 0
        nonterminals = {nonterminal}
        follow = set()
        while len(follow) + len(nonterminals) != old_length:
            old_length = len(follow) + len(nonterminals)
            for production in self.productions:
                for index in [index for index, item in enumerate(production.pattern) if item in nonterminals]:
                    if(index < len(production.pattern) - 1):
                        starts = {production.pattern[index + 1]}  # Some of these might be terminals, which will be left as-is. 
                        #Others will be nonterminals, which we need to figure out what possible terminals they could start with to find what possible terminals could follow the original query nonterminal. 
                        while True:
                            nont_starts = [i for i in starts if type(i) is tuple] # All nonterminals from starts
                            starts = set([i for i in starts if type(i) is not tuple]) # Filter starts to only contain terminals.
                            for production in [p for p in self.productions if p.nonterminal in nont_starts and len(p.pattern) > 0]:
                                starts.add(production.pattern[0])
                            if(len(nont_starts) is 0):
                                break
                        follow = follow.union(starts)
                    else:
                        nonterminals.add(production.nonterminal)
        return follow
    
    def create_dfa(self, start_nonterminal, eof_symbol = 'EOF'):
        self.create_production(('__TREE_ROOT__',), start_nonterminal, eof_symbol)
        nfa = NondeterminateFiniteAutomaton()
        prod_starts = []
        start_production = None
        for i, production in enumerate(self.productions):
            segment = production.create_nfa_segment(i)
            offset = nfa.add_states(segment)
            prod_starts.append(segment.start_state + offset)
            if(production.nonterminal == start_nonterminal):
                start_production = i
        for state in nfa.get_states():
            for trigger, end_states in [i for i in nfa.get_transitions_from(state) if type(i[0]) is tuple]: # Iterate over all transitions on nonterminals.
                for i in [i for i, p in enumerate(self.productions) if p.nonterminal == trigger]: # Add epsilon transitions from the starting point of the transition to the NFA segments that detect that nonterminal.
                    nfa.add_transition(state, '\x00', prod_starts[i])
        nfa.start_state = prod_starts[start_production]
        dfa = nfa.convert_to_dfa() # Now we have a DFA that will be satisfied whenever it reaches the end of a pattern.
        print(nfa, dfa)
        # It now needs to be converted to a stack DFA so that once it is satisfied, it will collapse to a nonterminal once it reaches the end of a pattern.
        cfgfa = CFGFiniteAutomaton(self)
        [cfgfa.add_state() for i in dfa.get_states()]
        end_state = None
        for state in dfa.get_states():
            for trigger, end_state in dfa.get_transitions_from(state):
                # If it is a nonterminal, just go to another state. It is not reading a terminal off the input.
                cfgfa.set_transition(state, trigger, [TRANSITION_SHIFT, TRANSITION_GO][type(trigger) is tuple], end_state)
            if(len(dfa.get_state_labels(state)) > 0): # Labels are used to indicate end points for productions. The label is the index of the production that the state completes.
                assert len(dfa.get_state_labels(state)) == 1 # There should only be one production the state can resolve to.
                production_index = list(dfa.get_state_labels(state))[0]
                production = self.productions[production_index]
                follows = self.follow(production.nonterminal)
                for follow in follows: # When the DFA has reached the end of a production's pattern, and a terminal that follows that pattern is found, the pattern should be reduced to the production's nonterminal.
                    previous_transition = cfgfa.transitions[state].get(follow) # .get() will return None if the key does not exist.
                    # Figure out if the follow key is a stronger operator than what is in the current production.
                    # If so, do not override it with a reduce. Keep the shift so that the stronger operator will reduce first and maintain proper order of operations.
                    if not (previous_transition and previous_transition[0] == TRANSITION_SHIFT and production.is_binary_operator 
                            and (self.productions[self.operators[follow]].tier < production.tier)): # Lower tier is more important (1st priority is more important than 2nd priority) 
                        cfgfa.set_transition(state, follow, TRANSITION_REDUCE, production_index)
                    print(previous_transition, '->', cfgfa.transitions[state].get(follow))
                        
        cfgfa.start_state = dfa.start_state
        cfgfa.set_transition(cfgfa.start_state, start_nonterminal, TRANSITION_ACCEPT, -1)
        del self.productions[-1]
        return cfgfa
    
    def __str__(self):
        return '\n'.join([str(i) for i in self.productions])
    
    def __repr__(self):
        return str(self)

class Production:
    def __init__(self, grammar, nonterminal, *pattern, **kwargs):
        self.parent = grammar
        if(type(nonterminal) is not tuple):
            nonterminal = (nonterminal,)
        self.nonterminal = nonterminal
        self.pattern = pattern
        self.is_binary_operator = kwargs.get('is_binary_operator', False)
        self.assoc = kwargs.get('assoc', ASSOC_LEFT)
        self.tier = kwargs.get('tier', 0)
    
    def __str__(self):
        # ID -> name
        return str(self.nonterminal) + ' -> ' + ' '.join([str(i) for i in self.pattern])
    
    def __repr__(self):
        return str(self)
    
    def create_nfa_segment(self, end_label):
        nfa = NondeterminateFiniteAutomaton()
        state = nfa.add_state()
        nfa.start_state = state
        for trigger in self.pattern:
            new_state = nfa.add_state()
            nfa.add_transition(state, trigger, new_state)
            state = new_state
        nfa.add_state_label(state, end_label)
        return nfa

if __name__ == '__main__':
    cfg = ContextFreeGrammar()
    '''
    cfg.create_production(('A',), '{', ('A',), '}')
    cfg.create_production(('A',), 'hello')
    cfg.create_production(('B',), 'behold: ', ('B',))
    cfg.create_production(('B',), ('A',))
    cfg.create_production(('START',), ('B',), 'EOF')
    '''
    cfg.create_production(('START',), ('T',))
    cfg.create_production(('T',), ('R2',))
    cfg.create_production(('T',), 'a', ('T',), 'c')
    cfg.create_production(('R2',), ('R2',), 'b', ('R',))
    cfg.create_production(('R2',), ('R',))
    cfg.create_production(('R',), 'p')
    print(cfg)
    cfgfa = cfg.create_dfa(('START',), '$')
    print(cfgfa)
    print(cfgfa.test('pbpbp$'))
    '''
    ast = AbstractSyntaxTree('ROOT')
    ast_a = AbstractSyntaxTree('A')
    ast_b = AbstractSyntaxTree('B')
    ast_c = AbstractSyntaxTree('C')
    ast.add_child(ast_a)
    ast.add_child(ast_b)
    ast_b.add_child(ast_c)
    ast_b.add_child(ast_c)
    print(ast)'''
    