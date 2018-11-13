#include "grammar/DataTypes.h"

#include <cassert>
#include <cmath>

#include "grammar/Expressions.h"
#include "intermediate/DataType.h"
#include "intermediate/Value.h"

namespace waveguide {
namespace convert {

DTypeSP grammar::NamedDataType::convert(ScopeSP context) {
    assert(context->lookupType(name));
    return context->lookupType(name);
}

DTypeSP grammar::ArrayDataType::convert(ScopeSP context) {
    assert(baseType);
    assert(size);
    ValueSP sizec = size->getValue(context);
    if (sizec->isValueKnown()) {
        int sizei = 0;
        if (std::dynamic_pointer_cast<intermediate::FloatDataType>(
            sizec->getType())) {
            sizei = floor(*sizec->dataAsFloat());
        } else if (std::dynamic_pointer_cast<intermediate::IntDataType>(
            sizec->getType())) {
            sizei = *sizec->dataAsInt();
        } else {
            // TODO: Error for non-numeric size;
            return nullptr;
        }
        return std::shared_ptr<intermediate::ArrayDataType>(
            new intermediate::ArrayDataType(baseType->convert(context), sizei)
        );
    } else {
        // TODO: Error for non-constant size;
        return nullptr;
    }
}

}
}