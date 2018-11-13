#include "DataTypes.h"

namespace waveguide {
namespace grammar {

////////////////////////////////////////////////////////////////////////////////
// NamedDataType, ArrayDataType
////////////////////////////////////////////////////////////////////////////////
NamedDataType::NamedDataType(std::string name)
    : name{name} { }

ArrayDataType::ArrayDataType(std::shared_ptr<DataType> baseType,
    std::shared_ptr<Expression> size)
    : baseType{baseType}, size{size} { }

}
}