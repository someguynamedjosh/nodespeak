#include <waveguide/intermediate/data_type.hpp>

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// data_type
////////////////////////////////////////////////////////////////////////////////
data_type::data_type() { }

std::shared_ptr<data_type> data_type::get_base_type() {
    return std::shared_ptr<data_type>(this);
}

bool data_type::is_proxy_type() {
    return false;
}

////////////////////////////////////////////////////////////////////////////////
// abstract_data_type
////////////////////////////////////////////////////////////////////////////////
abstract_data_type::abstract_data_type(std::string label)
    : label{label} { }

int abstract_data_type::get_length() {
    return 0;
}

std::string abstract_data_type::repr() {
    return label;
}

std::string abstract_data_type::format(void *data) {
    return "???";
}

////////////////////////////////////////////////////////////////////////////////
// int_data_type
////////////////////////////////////////////////////////////////////////////////
int int_data_type::get_length() {
    return 4;
}

std::string int_data_type::repr() {
    return "Int";
}

std::string int_data_type::format(void *data) {
    return std::to_string(*((int *) data));
}

////////////////////////////////////////////////////////////////////////////////
// float_data_type
////////////////////////////////////////////////////////////////////////////////
int float_data_type::get_length() {
    return 4;
}

std::string float_data_type::repr() {
    return "Float";
}

std::string float_data_type::format(void *data) {
    return std::to_string(*((float *) data));
}

////////////////////////////////////////////////////////////////////////////////
// bool_data_type
////////////////////////////////////////////////////////////////////////////////
int bool_data_type::get_length() {
    return 1;
}

std::string bool_data_type::repr() {
    return "Bool";
}

std::string bool_data_type::format(void *data) {
    return (char *) data != 0 ? "true" : "false";
}

////////////////////////////////////////////////////////////////////////////////
// array_data_type
////////////////////////////////////////////////////////////////////////////////
array_data_type::array_data_type(std::shared_ptr<data_type> elementType, int length)
    : elementType{elementType}, length{length} { }

int array_data_type::get_length() {
    return elementType->get_length() * length;
}

std::shared_ptr<data_type> array_data_type::get_base_type() {
    return elementType->get_base_type();
}

std::string array_data_type::repr() {
    return elementType->repr() + "[" + std::to_string(length) + "]";
}

std::string array_data_type::format(void *data) {
    std::string tr = "[";
    for (int i = 0; i < length; i++) {
        tr += elementType->format(data + i * elementType->get_length());
        if (i != length - 1) {
            tr += ", ";
        }
    }
    return tr + "]";
}

int array_data_type::getArrayLength() {
    return length;
}

std::shared_ptr<data_type> array_data_type::get_element_type() {
    return elementType;
}

std::shared_ptr<value> array_data_type::get_data_offset(std::shared_ptr<value> index) {
    // TODO: Implementation
    return std::shared_ptr<value>{nullptr};
}

////////////////////////////////////////////////////////////////////////////////
// copy_array_data_proxy
////////////////////////////////////////////////////////////////////////////////
copy_array_data_proxy::copy_array_data_proxy(std::shared_ptr<data_type> sourceType, int length)
    : array_data_type{sourceType, length} { }

bool copy_array_data_proxy::is_proxy() {
    return true;
}

std::string copy_array_data_proxy::format(void *data) {
    std::string tr = "[";
    for (int i = 0; i < getArrayLength(); i++) {
        tr += get_element_type()->format(data);
        if (i != getArrayLength() - 1) {
            tr += ", ";
        }
    }
    return tr + "]";
}

std::string copy_array_data_proxy::repr() {
    return get_element_type()->repr() + "[" + std::to_string(getArrayLength()) 
        + " copied from 1]";
}

std::shared_ptr<value> copy_array_data_proxy::get_data_offset(std::shared_ptr<value> index) {
    // TODO: Implementation
    return std::shared_ptr<value>{nullptr};
}

}
}
