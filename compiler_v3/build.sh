#!/bin/zsh
echo "===================================================="
bison -v -d waveguide.y; flex waveguide.l
g++ -g waveguide.tab.c lex.yy.c tokens.cpp scope.cpp -lfl -o waveguide 2>&1 | head -n 20
