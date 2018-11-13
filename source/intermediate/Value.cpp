#include "Value.h"

#include <cassert>
#include <cstring>
#include <sstream>
#include <string>

#include "DataType.h"

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// Com::Value
////////////////////////////////////////////////////////////////////////////////

Value::Value(std::shared_ptr<DataType> type)
    : type{type} {
    if (!type->isProxyType()) {
        data = malloc(type->getLength());
    }
}

Value::Value(std::shared_ptr<DataType> type, void *data)
    : type{type}, data{data}, valueKnown{true} { }

Value::~Value() {
    if (!type->isProxyType()) {
        free(data);
    }
}

std::string Value::repr() {
    std::stringstream ss;
    ss << isValueKnown() ? "C" : "V";
    ss << "@" << (void*) this;
    ss << " T=" << type->repr();
    if (isValueKnown()) {
        ss << " V=" << type->format(getRealValue().getData());
    }
    return ss.str();
}

std::shared_ptr<DataType> Value::getType() {
    return type;
}

void Value::setType(std::shared_ptr<DataType> newType) {
    assert(newType->getLength() == type->getLength());
    assert(newType->isProxyType() == type->isProxyType());
    type = newType;
}

bool Value::isProxy() {
    return type->isProxyType();
}

Value &Value::getRealValue() {
    if (isProxy) {
        return static_cast<Value*>(data)->getRealValue();
    } else {
        return *this;
    }
}

bool Value::isValueKnown() {
    return valueKnown;
}

void Value::setValueKnown(bool isKnown) {
    valueKnown = isKnown;
}

Value Value::createKnownCopy() {
    assert(valueKnown);
    Value tr{type};
    memcpy(tr.getData(), data, type->getLength());
    tr.setValueKnown(true);
    return tr;
}

void *Value::getData() {
    assert(!isProxy());
    return data;
}

float *Value::dataAsFloat() {
    assert(!isProxy());
    assert(std::dynamic_pointer_cast<FloatDataType>(type));
    return static_cast<float*>(data);
}

int *Value::dataAsInt() {
    assert(!isProxy());
    assert(std::dynamic_pointer_cast<IntDataType>(type));
    return static_cast<int*>(data);
}

bool *Value::dataAsBool() {
    assert(!isProxy());
    assert(std::dynamic_pointer_cast<BoolDataType>(type));
    return static_cast<bool*>(data);
}

}
}