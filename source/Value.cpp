#include "Value.h"

#include <cassert>
#include <cstring>
#include <string>

#include "DataType.h"

namespace Com {

////////////////////////////////////////////////////////////////////////////////
// Com::Value
////////////////////////////////////////////////////////////////////////////////

Value::Value(DataType &type)
    : type{type} {
    if (!type.isProxyType()) {
        data = malloc(type.getLength());
    }
}

Value::Value(DataType &type, void *data)
    : type{type}, data{data}, valueKnown{true} { }

Value::~Value() {
    if (!type.isProxyType()) {
        free(data);
    }
}

std::string Value::format() {
    return type.format(getRealValue().getData());
}

DataType &Value::getType() {
    return type;
}

void Value::setType(DataType &newType) {
    assert(newType.getLength() == type.getLength());
    assert(newType.isProxyType() == type.isProxyType());
    type = newType;
}

bool Value::isProxy() {
    return type.isProxyType();
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
    memcpy(tr.getData(), data, type.getLength());
    tr.setValueKnown(true);
    return tr;
}

void *Value::getData() {
    assert(!isProxy());
    return data;
}

float *Value::dataAsFloat() {
    assert(!isProxy());
    assert(dynamic_cast<FloatDataType*>(&type));
    return static_cast<float*>(data);
}

int *Value::dataAsInt() {
    assert(!isProxy());
    assert(dynamic_cast<IntDataType*>(&type));
    return static_cast<int*>(data);
}

bool *Value::dataAsBool() {
    assert(!isProxy());
    assert(dynamic_cast<BoolDataType*>(&type));
    return static_cast<bool*>(data);
}

}