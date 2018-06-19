bison -v -d waveguide.y; flex waveguide.l; g++ waveguide.tab.c lex.yy.c -lfl -o waveguide 2>&1 | head -n 20
