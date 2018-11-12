INCLUDE = -I $(SRC) -I $(GEN)
CXX = g++
CXXFLAGS = -g $(INCLUDE)

BIN = build/bin/
GEN = build/gen/
OBJ = build/obj/
DEP = build/deps/
SRC = source/
# Create the build folders. Do it now, unconditionally, so that if Make decides to generate and import the .dep files,
# they will have a folder to be written to.
$(shell mkdir -p build)
$(shell mkdir -p $(BIN))
$(shell mkdir -p $(DEP))
$(shell mkdir -p $(GEN))
$(shell mkdir -p $(OBJ))

GRAMMAR_C = $(GEN)parser.gen.c
GRAMMAR_H = $(GEN)parser.gen.h
LEXER_C = $(GEN)lexer.gen.c
OUTPUT = $(BIN)waveguide.x86_64

SRCS = $(wildcard $(SRC)*.cpp)
OBJS = $(patsubst $(SRC)%.cpp,$(OBJ)%.o,$(SRCS))
DEPS = $(patsubst $(SRC)%.cpp,$(DEP)%.dep,$(SRCS))

GSRCS = $(GRAMMAR_C) $(LEXER_C)
GOBJS = $(patsubst $(GEN)%.gen.c,$(OBJ)%.gen.o,$(GSRCS))
GDEPS = $(patsubst $(GEN)%.gen.c,$(DEP)%.gen.dep,$(GSRCS))

all: $(OUTPUT) 

# Build final executable from all object files.
$(OUTPUT): $(OBJS) $(GOBJS)
	$(info >>> Linking executable.)
	$(CXX) $(CXXFLAGS) $(OBJS) $(GOBJS) -lfl -o $@
	chmod +x $(BIN)waveguide.x86_64
	@echo Build completed sucessfully!

# Generate dependency files (lists of headers that are included) for each file, to automate dependency tracking.
# Note that this method does not include the most recent changes, but this is not necessary. If a file has modified its
# dependencies, it will already be rebuilt anyway because the code might have changed too.
-include $(GDEPS)
-include $(DEPS)
$(DEP)%.dep: $(SRC)%.cpp
	$(info >>> Generating dependencies for $<.)
	@$(CXX) $(CXXFLAGS) $< -MM -MT $(OBJ)$*.o -o $@

# Build auto-generated sources.
$(OBJ)%.gen.o: $(GEN)%.gen.c # Other dependencies are added automatically.
	$(eval SOURCE := $(patsubst $(OBJ)%.gen.o,$(GEN)%.gen.c,$@))
	$(info >>> Building generated file $(SOURCE).)
	$(CXX) $(CXXFLAGS) -c $(SOURCE) -o $@
# Build regular sources.
$(OBJ)%.o: # Dependencies are added automatically.
	$(eval SOURCE := $(patsubst $(OBJ)%.o,$(SRC)%.cpp,$@))
	$(info >>> Building file $(SOURCE).)
	$(CXX) $(CXXFLAGS) -c $(SOURCE) -o $@

# Generate files for lexer and parser from their .l and .y files.
$(GRAMMAR_C) $(GRAMMAR_H): $(SRC)waveguide.y
	$(info >>> Generating parser with bison.)
	bison -v --defines="$(GRAMMAR_H)" --output="$(GRAMMAR_C)" $(SRC)waveguide.y;
	# Generate dependency file.
	@$(CXX) $(CXXFLAGS) $(GRAMMAR_C) -MM -MT $(patsubst $(GEN)%.gen.c,$(OBJ)%.gen.o,$(GRAMMAR_C)) \
		-o $(patsubst $(GEN)%.gen.c,$(DEP)%.gen.dep,$(GRAMMAR_C))
$(LEXER_C): $(SRC)waveguide.l
	$(info >>> Generating lexer with flex.)
	flex --outfile="$(LEXER_C)" $(SRC)waveguide.l;
	# Generate dependency file.
	@$(CXX) $(CXXFLAGS) $(GRAMMAR_C) -MM -MT $(patsubst $(GEN)%.gen.c,$(OBJ)%.gen.o,$(LEXER_C)) \
		-o $(patsubst $(GEN)%.gen.c,$(DEP)%.gen.dep,$(LEXER_C))

clean:
	@rm ${BIN}*
	@rm ${GEN}*
	@rm ${OBJ}*
	@rm ${DEP}*

.PHONY: all clean