from finite_automata import NondeterminateFiniteAutomaton

def r_or(*options):
    if(type(options[0]) is list and len(options) is 1):
        options = options[0]
    return ('or',) + options

def n_or(regexp, nfa): # Converts a regular expression to an NFA segment.
    assert regexp[0] == 'or'
    start_state, end_state = nfa.add_state(), nfa.add_state()
    for option in regexp[1:]:
        if(type(option) is str): # One or more characters
            if(option == ''): # Empty transition
                nfa.add_transition(start_state, '', end_state)
            for char in option:
                nfa.add_transition(start_state, char, end_state)
        elif(type(option) is tuple and type(option[1]) is tuple): # A regexp segment that has been converted to a series of NFA states.
            nfa.add_transition(start_state, '', option[1][0]) # Add a blank transition to the start of the option's network.
            nfa.add_transition(option[1][1], '', end_state) # Blank transition from the output of the option's network to the overall output.
    return regexp[:1] + ((start_state, end_state),) + regexp[1:]

def r_and(*sequence):
    if(type(sequence[0]) is tuple and len(sequence) == 1):
        sequence = sequence[0]
    return ('and',) + sequence

def n_and(regexp, nfa):
    assert regexp[0] == 'and'
    start_state = nfa.add_state()
    middle_state = start_state
    for option in regexp[1:]:
        if(type(option) is str): # One or more characters
            for char in option:
                nms = nfa.add_state()
                nfa.add_transition(middle_state, char, nms)
                middle_state = nms
        elif(type(option) is tuple and type(option[1]) is tuple): # A regexp segment that has been converted to a series of NFA states.
            nfa.add_transition(middle_state, '', option[1][0]) # Add a blank transition to the start of the option's network.
            middle_state = option[1][1]
    return regexp[:1] + ((start_state, middle_state),) + regexp[1:]

def r_repeat(regexp, minimum_repeats=0, maximum_repeats=None):
    out = ['and'] + [regexp] * minimum_repeats
    if(maximum_repeats is not None and maximum_repeats > minimum_repeats):
        sub = ('or', regexp, '')
        for i in range(maximum_repeats - minimum_repeats - 1):
            sub = ('or', ('and', regexp, sub), '')
        out.append(sub)
    else:
        out.append(('repeat', regexp))
    if(len(out) == 2): # And [single element
        out = out[1]
    return tuple(out)

def n_repeat(regexp, nfa):
    assert regexp[0] == 'repeat'
    assert type(regexp[1][1]) is tuple
    loop_state = nfa.add_state()
    nfa.add_transition(loop_state, '', regexp[1][1][0])
    nfa.add_transition(regexp[1][1][1], '', loop_state)
    return regexp[:1] + ((loop_state, loop_state),) + regexp[1:]

def gen_range(min_char, max_char):
    return [chr(i) for i in range(ord(min_char), ord(max_char) + 1)]

SET_LOWERCASE_LETTERS = gen_range('a', 'z')
SET_UPPERCASE_LETTERS = gen_range('A', 'Z')
SET_LETTERS = SET_LOWERCASE_LETTERS + SET_UPPERCASE_LETTERS
SET_NUMBERS = gen_range('0', '9')
SET_ALPHANUMERIC = SET_LETTERS + SET_NUMBERS

def regexp_to_nfa(regexp, output_label=1):
    nfa = NondeterminateFiniteAutomaton()
    def internal_recursor(regexp):
        # Make sure sub expressions have been converted to NFAs
        converted = (regexp[0],) # Element 0 denotes the function (and, or, repeat) of the expression.
        for sub in regexp[1:]:
            if(type(sub) is tuple):
                converted += (internal_recursor(sub),)
            else: # Just a normal string constant.
                converted += (sub,)
        return {
            'and': n_and,
            'or': n_or,
            'repeat': n_repeat
        }[regexp[0]](converted, nfa)
    nfad = internal_recursor(regexp)[1]
    nfa.set_state_label(nfad[0], 0) # Starting point.
    nfa.set_state_label(nfad[1], output_label) # Exit point.
    return nfa
    
if __name__ == '__main__':
    exp = r_or('a', 'b', r_and('fgh', r_or(r_and('hello world'), r_and('how are you doing')))) # Regexp for numbers.
    print(exp)
    nfa = regexp_to_nfa(exp)
    print(nfa)
    print(nfa.convert_to_dfa())