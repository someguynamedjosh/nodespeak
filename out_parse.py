import sys

tokens = {}
for line in sys.stdin:
    if (' is ' in line and ' is Scope' not in line):
        sline = line.strip()
        name = sline[sline.find(' is ')+3:]
        if (name[-1] == ':'):
            name = name[:-1]
        hexv = sline[:sline.find(' is ')].strip()
        tokens[hexv] = name
    line = line.rstrip()
    for key, value in tokens.items():
        line = line.replace(key, value)
    print(line)
