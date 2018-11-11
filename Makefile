INCLUDE = -I $(SRC) -I $(GEN)
CXX = g++
CXXFLAGS = -g $(INCLUDE)

BIN = build/bin/
GEN = build/gen/
OBJ = build/obj/
DEP = build/deps/
SRC = source/

GRAMMAR_C = $(GEN)parser.gen.c
GRAMMAR_H = $(GEN)parser.gen.h
LEXER_C = $(GEN)lexer.gen.c
OUTPUT = $(BIN)waveguide.x86_64

SRCS = $(wildcard $(SRC)*.cpp)
OBJS = $(patsubst $(SRC)%.cpp,$(OBJ)%.o,$(SRCS))
DEPS = $(patsubst $(SRC)%.cpp,$(DEP)%.dep,$(SRCS))

GSRCS = $(GRAMMAR_C) $(LEXER_C)
$(info $(GSRCS))
$(info $(patsubst %.c,%.o,$(GSRCS)))
GOBJS = $(patsubst $(GEN)%.gen.c,$(OBJ)%.gen.o,$(GSRCS))
GDEPS = $(patsubst $(GEN)%.gen.c,$(DEP)%.gen.dep,$(GSRCS))

all: base $(OUTPUT) 

# Build final executable from all object files.
$(OUTPUT): $(OBJS) $(GOBJS)
	$(info $(GOBS))
	$(CXX) $(CXXFLAGS) $(OBJS) -lfl -o $@
	chmod +x $(BIN)waveguide.x86_64

# Generate dependency files (lists of headers that are included) for each file, to automate dependency tracking.
# Note that this method does not include the most recent changes, but this is not necessary. If a file has modified its
# dependencies, it will already be rebuilt anyway because the code might have changed too.
-include $(GDEPS)
$(DEP)%.gen.dep: grammar $(GEN)%.gen.cpp
	@$(CXX) $(CXXFLAGS) $< -MM -MT $(OBJ)$*.gen.o -o $@
-include $(DEPS)
$(DEP)%.dep: $(SRC)%.cpp
	@$(CXX) $(CXXFLAGS) $< -MM -MT $(OBJ)$*.o -o $@

# Build auto-generated sources.
$(OBJ)%.gen.o: grammar # Dependencies are added automatically.
	$(CXX) $(CXXFLAGS) -c $(patsubst $(OBJ)%.gen.o,$(GEN)%.gen.c,$@) -o $@
# Build regular sources.
$(OBJ)%.o: # Dependencies are added automatically.
	$(CXX) $(CXXFLAGS) -c $(patsubst $(OBJ)%.o,$(SRC)%.cpp,$@) -o $@

# Generate files for lexer and parser from their .l and .y files.
grammar: $(GRAMMAR_C) $(GRAMMAR_H) $(LEXER_C)
$(GRAMMAR_C) $(GRAMMAR_H): $(SRC)waveguide.y
	bison -v --defines="$(GRAMMAR_H)" --output="$(GRAMMAR_C)" $(SRC)waveguide.y;
$(LEXER_C): $(SRC)waveguide.l
	flex --outfile="$(LEXER_C)" $(SRC)waveguide.l;

# Generate the output tree structure.
base: build $(BIN) $(DEP) $(GEN) $(OBJ)
$(BIN): build
	mkdir -p $(BIN)
$(DEP): build
	mkdir -p $(DEP)
$(GEN): build
	mkdir -p $(GEN)
$(OBJ): build
	mkdir -p $(OBJ)
build:
	mkdir -p build

.PHONY: all grammar base