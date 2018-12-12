import sys

tokens = {}
for line in sys.stdin:
    if (' is ' in line and ' is Scope' not in line):
        name = line[line.find(' is ')+3:]
        if (name[-1] == ':'):
            name = name[;-1]
        hexv = line[:line.find(' is ')].strip()
        tokens[hexv] = name
    else:
        for key, value in tokens.items():
            line = line.replace(key, value)
        print(line)
