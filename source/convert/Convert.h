#ifndef _CONVERT_CONVERT_H
#define _CONVERT_CONVERT_H

#include "Utils.h"

namespace waveguide {
namespace grammar {

class StatList;

}
}

namespace waveguide {
namespace convert {

ScopeSP convertSyntaxTree(std::shared_ptr<grammar::StatList> tree);

}
}

#endif /* _CONVERT_CONVERT_H */