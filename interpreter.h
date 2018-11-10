#include <vector>

#include "scope.h"

namespace Com {

DataType *pickCastType(DataType *a, DataType *b);
void interpret(Scope *root);
void runFuncScope(FuncScope *fs, std::vector<Value*> inputs, std::vector<Value*> outputs);

};

