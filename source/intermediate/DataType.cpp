#include "DataType.h"

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// DataType
////////////////////////////////////////////////////////////////////////////////
DataType::DataType() { }

DataType &DataType::getBaseType() {
    return *this;
}

bool DataType::isProxyType() {
    return false;
}

////////////////////////////////////////////////////////////////////////////////
// AbstractDataType
////////////////////////////////////////////////////////////////////////////////
AbstractDataType::AbstractDataType(std::string label)
    : label{label} { }

int AbstractDataType::getLength() {
    return 0;
}

std::string AbstractDataType::repr() {
    return label;
}

std::string AbstractDataType::format(void *data) {
    return "???";
}

////////////////////////////////////////////////////////////////////////////////
// IntDataType
////////////////////////////////////////////////////////////////////////////////
int IntDataType::getLength() {
    return 4;
}

std::string IntDataType::repr() {
    return "Int";
}

std::string IntDataType::format(void *data) {
    return std::to_string(*((int *) data));
}

////////////////////////////////////////////////////////////////////////////////
// FloatDataType
////////////////////////////////////////////////////////////////////////////////
int FloatDataType::getLength() {
    return 4;
}

std::string FloatDataType::repr() {
    return "Float";
}

std::string FloatDataType::format(void *data) {
    return std::to_string(*((float *) data));
}

////////////////////////////////////////////////////////////////////////////////
// BoolDataType
////////////////////////////////////////////////////////////////////////////////
int BoolDataType::getLength() {
    return 1;
}

std::string BoolDataType::repr() {
    return "Bool";
}

std::string BoolDataType::format(void *data) {
    return (char *) data != 0 ? "true" : "false";
}

////////////////////////////////////////////////////////////////////////////////
// ArrayDataType
////////////////////////////////////////////////////////////////////////////////
ArrayDataType::ArrayDataType(DataType &elementType, int length)
    : elementType{elementType}, length{length} { }

int ArrayDataType::getLength() {
    return elementType.getLength() * length;
}

DataType &ArrayDataType::getBaseType() {
    return elementType.getBaseType();
}

int ArrayDataType::getArrayLength() {
    return length;
}

DataType &ArrayDataType::getElementType() {
    return elementType;
}

std::string ArrayDataType::repr() {
    return elementType.repr() + "[" + std::to_string(length) + "]";
}

std::string ArrayDataType::format(void *data) {
    std::string tr = "[";
    for (int i = 0; i < length; i++) {
        tr += elementType.format(data + i * elementType.getLength());
        if (i != length - 1) {
            tr += ", ";
        }
    }
    return tr + "]";
}

////////////////////////////////////////////////////////////////////////////////
// CopyArrayDataProxy
////////////////////////////////////////////////////////////////////////////////
CopyArrayDataProxy::CopyArrayDataProxy(DataType &sourceType, int length)
    : ArrayDataType{sourceType, length} { }

bool CopyArrayDataProxy::isProxy() {
    return true;
}

std::string CopyArrayDataProxy::format(void *data) {
    std::string tr = "[";
    for (int i = 0; i < getArrayLength(); i++) {
        tr += getElementType().format(data);
        if (i != getArrayLength() - 1) {
            tr += ", ";
        }
    }
    return tr + "]";
}

std::string CopyArrayDataProxy::repr() {
    return getElementType().repr() + "[" + std::to_string(getArrayLength()) 
        + " copied from 1]";
}

}
}
