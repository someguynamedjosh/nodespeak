import sys

tokens = {}
for line in sys.stdin:
    if (line[:5] == 'type ' or line[:5] == 'func ' or line[:4] == 'var '):
        name = line[line.find(' ')+1 : line.find(':')]
        hexv = line[line.find(':')+2:-1]
        tokens[hexv] = name
    if (line[:6] == 'funcc ' or line[:2] == '=='):
        for key, value in tokens.items():
            line = line.replace(key, value)
        print(line)
