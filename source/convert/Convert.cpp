#include "Convert.h"

#include "grammar/Statements.h"

namespace waveguide {
namespace convert {

void parseStatList(ScopeSP context, std::shared_ptr<grammar::StatList> list) {
    for (auto stat : list->getStatements()) {
        stat->convert(context);
    }

    // Only parse function bodies after everything has been declared.
    // (Function hoisting.)
    for (auto stat : list->getStatements()) {
        if (auto fdec = std::dynamic_pointer_cast<grammar::FuncDec>(stat)) {
            parseStatList(context->lookupFunc(fdec->getName()), 
                fdec->getBody());
        }
    }
}

ScopeSP convertSyntaxTree(std::shared_ptr<grammar::StatList> tree) {
    ScopeSP root{new Scope()};
    blt()->addToScope(root);
    parseStatList(root, tree);
    return root;
}

}
}