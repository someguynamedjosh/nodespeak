import sys

tokens = {}
for line in sys.stdin:
    line = line.rstrip()
    if (' is ' in line and ' is Scope' not in line and 'Lambda' not in line):
        sline = line.strip()
        name = sline[sline.find(' is ')+3:].strip()
        if (name[-1] == ':'):
            name = name[:-1]
        hexv = sline[:sline.find(' is ')].split()[-1].strip()
        tokens[hexv] = name
    else:
        for key, value in tokens.items():
            line = line.replace(key, value)
    print(line)
