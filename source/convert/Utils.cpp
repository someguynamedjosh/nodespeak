#include "Utils.h"

namespace waveguide {
namespace convert {

int getDataTypeIndex(DTypeSP t) {
    if (std::dynamic_pointer_cast<intermediate::BoolDataType>(t)) {
        return 10;
    } else if (std::dynamic_pointer_cast<intermediate::IntDataType>(t)) {
        return 20;
    } else if (std::dynamic_pointer_cast<intermediate::FloatDataType>(t)) {
        return 30;
    }
}

DTypeSP pickBiggerType(DTypeSP a, DTypeSP b) {
    return getDataTypeIndex(a) > getDataTypeIndex(b) ? a : b;
}

}
}
