#include "data_type.hpp"

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// DataType
////////////////////////////////////////////////////////////////////////////////
DataType::DataType() { }

std::shared_ptr<DataType> DataType::get_base_type() {
    return std::shared_ptr<DataType>(this);
}

bool DataType::is_proxy_type() {
    return false;
}

////////////////////////////////////////////////////////////////////////////////
// AbstractDataType
////////////////////////////////////////////////////////////////////////////////
AbstractDataType::AbstractDataType(std::string label)
    : label{label} { }

int AbstractDataType::get_length() {
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
int IntDataType::get_length() {
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
int FloatDataType::get_length() {
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
int BoolDataType::get_length() {
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
ArrayDataType::ArrayDataType(std::shared_ptr<DataType> elementType, int length)
    : elementType{elementType}, length{length} { }

int ArrayDataType::get_length() {
    return elementType->get_length() * length;
}

std::shared_ptr<DataType> ArrayDataType::get_base_type() {
    return elementType->get_base_type();
}

int ArrayDataType::getArrayLength() {
    return length;
}

std::shared_ptr<DataType> ArrayDataType::get_element_type() {
    return elementType;
}

std::string ArrayDataType::repr() {
    return elementType->repr() + "[" + std::to_string(length) + "]";
}

std::string ArrayDataType::format(void *data) {
    std::string tr = "[";
    for (int i = 0; i < length; i++) {
        tr += elementType->format(data + i * elementType->get_length());
        if (i != length - 1) {
            tr += ", ";
        }
    }
    return tr + "]";
}

////////////////////////////////////////////////////////////////////////////////
// CopyArrayDataProxy
////////////////////////////////////////////////////////////////////////////////
CopyArrayDataProxy::CopyArrayDataProxy(std::shared_ptr<DataType> sourceType, int length)
    : ArrayDataType{sourceType, length} { }

bool CopyArrayDataProxy::is_proxy() {
    return true;
}

std::string CopyArrayDataProxy::format(void *data) {
    std::string tr = "[";
    for (int i = 0; i < getArrayLength(); i++) {
        tr += get_element_type()->format(data);
        if (i != getArrayLength() - 1) {
            tr += ", ";
        }
    }
    return tr + "]";
}

std::string CopyArrayDataProxy::repr() {
    return get_element_type()->repr() + "[" + std::to_string(getArrayLength()) 
        + " copied from 1]";
}

}
}
