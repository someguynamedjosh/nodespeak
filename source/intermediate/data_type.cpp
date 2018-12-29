#include <waveguide/intermediate/data_type.hpp>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// data_type
////////////////////////////////////////////////////////////////////////////////
data_type::data_type() { }

std::shared_ptr<const data_type> data_type::get_base_type() const {
    return std::shared_ptr<const data_type>(this);
}

bool data_type::is_proxy_type() const {
    return false;
}

////////////////////////////////////////////////////////////////////////////////
// abstract_data_type
////////////////////////////////////////////////////////////////////////////////
abstract_data_type::abstract_data_type(std::string label)
    : label{label} { }

int abstract_data_type::get_length() const {
    return 0;
}

std::string abstract_data_type::repr() const {
    return label;
}

std::string abstract_data_type::format(const void *data) const {
    return "???";
}

////////////////////////////////////////////////////////////////////////////////
// unresolved_vague_type
////////////////////////////////////////////////////////////////////////////////
unresolved_vague_type::unresolved_vague_type(SP<vague_data_type> unresolved)
    : unresolved(unresolved) { }

SP<vague_data_type> unresolved_vague_type::get_vague_type() const {
    return unresolved;
}

int unresolved_vague_type::get_length() const {
    return 0;
}

bool unresolved_vague_type::is_proxy_type() const {
    return true;
}

std::string unresolved_vague_type::repr() const {
    return "Unresolved vague data type";
}

std::string unresolved_vague_type::format(const void *data) const {
    return "???";
}

////////////////////////////////////////////////////////////////////////////////
// int_data_type
////////////////////////////////////////////////////////////////////////////////
int int_data_type::get_length() const {
    return 4;
}

std::string int_data_type::repr() const {
    return "Int";
}

std::string int_data_type::format(const void *data) const {
    return std::to_string(*((int *) data));
}

////////////////////////////////////////////////////////////////////////////////
// float_data_type
////////////////////////////////////////////////////////////////////////////////
int float_data_type::get_length() const {
    return 4;
}

std::string float_data_type::repr() const {
    return "Float";
}

std::string float_data_type::format(const void *data) const {
    return std::to_string(*((float *) data));
}

////////////////////////////////////////////////////////////////////////////////
// bool_data_type
////////////////////////////////////////////////////////////////////////////////
int bool_data_type::get_length() const {
    return 1;
}

std::string bool_data_type::repr() const {
    return "Bool";
}

std::string bool_data_type::format(const void *data) const {
    return (char *) data != 0 ? "true" : "false";
}

////////////////////////////////////////////////////////////////////////////////
// array_data_type
////////////////////////////////////////////////////////////////////////////////
array_data_type::array_data_type(std::shared_ptr<const data_type> element_type, 
    int length)
    : element_type{element_type}, length{length} { }

int array_data_type::get_length() const {
    return element_type->get_length() * length;
}

std::shared_ptr<const data_type> array_data_type::get_base_type() const {
    return element_type->get_base_type();
}

std::string array_data_type::repr() const {
    return element_type->repr() + "[" + std::to_string(length) + "]";
}

std::string array_data_type::format(const void *data) const {
    std::string tr = "[";
    for (int i = 0; i < length; i++) {
        tr += element_type->format(data + i * element_type->get_length());
        if (i != length - 1) {
            tr += ", ";
        }
    }
    return tr + "]";
}

int array_data_type::get_array_length() const {
    return length;
}

std::shared_ptr<const data_type> array_data_type::get_element_type() const {
    return element_type;
}

std::shared_ptr<value> 
    array_data_type::get_data_offset(std::shared_ptr<value> index)  const {
    // TODO: Implementation
    return std::shared_ptr<value>{nullptr};
}

////////////////////////////////////////////////////////////////////////////////
// copy_array_data_proxy
////////////////////////////////////////////////////////////////////////////////
copy_array_data_proxy::copy_array_data_proxy(
    std::shared_ptr<const data_type> source_type, int length)
    : array_data_type{source_type, length} { }

bool copy_array_data_proxy::is_proxy_type() const {
    return true;
}

std::string copy_array_data_proxy::format(const void *data) const {
    std::string tr = "[";
    for (int i = 0; i < get_array_length(); i++) {
        tr += get_element_type()->format(data);
        if (i != get_array_length() - 1) {
            tr += ", ";
        }
    }
    return tr + "]";
}

std::string copy_array_data_proxy::repr() const {
    return get_element_type()->repr() + "[" + std::to_string(get_array_length()) 
        + " copied from 1]";
}

std::shared_ptr<value> 
    copy_array_data_proxy::get_data_offset(std::shared_ptr<value> index) const {
    // TODO: Implementation
    return std::shared_ptr<value>{nullptr};
}

}
}
