INCLUDE = -I . -I build/gen/
CXX = g++
CXXFLAGS = -g ${INCLUDE}

BIN = build/bin/
GEN = build/gen/
OBJ = build/obj/
OBJS = ${OBJ}parser.o ${OBJ}lexer.o ${OBJ}interpreter.o ${OBJ}scope.o ${OBJ}tokens.o
GRAMMAR_C = ${GEN}waveguideGrammar.c
GRAMMAR_H = ${GEN}waveguideGrammar.h
LEXER = ${GEN}waveguideLexer.c
OUTPUT = ${BIN}waveguide.x86_64

all: base ${OUTPUT} 

${OUTPUT}: ${OBJS}
	${CXX} ${CXXFLAGS} -o $@ ${OBJS} -lfl
	chmod +x ${BIN}waveguide.x86_64

${OBJ}parser.o: ${GRAMMAR_C} ${GRAMMAR_H} tokens.h scope.h interpreter.h
	${CXX} ${CXXFLAGS} -c ${GRAMMAR_C} -o $@
${OBJ}lexer.o: ${LEXER} ${GRAMMAR_H} tokens.h
	${CXX} ${CXXFLAGS} -c ${LEXER} -o $@
${OBJ}interpreter.o: interpreter.cpp interpreter.h scope.h
	${CXX} ${CXXFLAGS} -c interpreter.cpp -o $@
${OBJ}scope.o: scope.cpp scope.h tokens.h
	${CXX} ${CXXFLAGS} -c scope.cpp -o $@
${OBJ}tokens.o: tokens.cpp tokens.h scope.h
	${CXX} ${CXXFLAGS} -c tokens.cpp -o $@

grammar: ${GRAMMAR_C} ${GRAMMAR_H} ${LEXER}
${GRAMMAR_C} ${GRAMMAR_H}: waveguide.y
	bison -v --defines="build/gen/waveguideGrammar.h" --output="build/gen/waveguideGrammar.c" waveguide.y;
${LEXER}: waveguide.l
	flex --outfile="build/gen/waveguideLexer.c" waveguide.l;

base: build ${BIN} ${GEN} ${OBJ}
${BIN}: build
	mkdir -p ${BIN}
${GEN}: build
	mkdir -p ${GEN}
${OBJ}: build
	mkdir -p ${OBJ}
build:
	mkdir -p build

.PHONY: all grammar base