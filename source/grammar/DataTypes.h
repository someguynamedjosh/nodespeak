#ifndef _WAVEGUIDE_GRAMMAR_DATA_TYPES_H_
#define _WAVEGUIDE_GRAMMAR_DATA_TYPES_H_

#include "Token.h"

namespace waveguide {
namespace grammar {

class Expression;

class DataType: public Token {
public:
    virtual convert::DTypeSP convert(convert::ScopeSP context) = 0;
};

class NamedDataType: public DataType {
private:
    std::string name;
public:
    NamedDataType(std::string name);
    virtual convert::DTypeSP convert(convert::ScopeSP context);
};

class ArrayDataType: public DataType {
private:
    std::shared_ptr<DataType> baseType;
    std::shared_ptr<Expression> size;
public:
    ArrayDataType(std::shared_ptr<DataType> baseType,
        std::shared_ptr<Expression> size);
    virtual convert::DTypeSP convert(convert::ScopeSP context);
};

}
}

#endif /* _WAVEGUIDE_GRAMMAR_DATA_TYPES_H_ */