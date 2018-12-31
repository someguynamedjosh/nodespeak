#include <waveguide/intermediate/data_type.hpp>

#include <ostream>
#include <waveguide/intermediate/type_template.hpp>

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

int data_type::get_array_depth() const {
    return 0;
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

void abstract_data_type::print_repr(std::ostream &stream) const {
    stream << label;
}

void abstract_data_type::format(std::ostream &stream, const void *data) const {
    stream << "??? at " << data;
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

void unresolved_vague_type::print_repr(std::ostream &stream) const {
    stream << "[UVDT] ";
    unresolved->print_repr(stream);
}

void unresolved_vague_type::format(std::ostream &stream, const void *data) const {
    stream << "??? at " << data;
}

////////////////////////////////////////////////////////////////////////////////
// int_data_type
////////////////////////////////////////////////////////////////////////////////
int int_data_type::get_length() const {
    return 4;
}

void int_data_type::print_repr(std::ostream &stream) const {
    stream << "Int";
}

void int_data_type::format(std::ostream &stream, const void *data) const {
    stream << *((int *) data);
}

////////////////////////////////////////////////////////////////////////////////
// float_data_type
////////////////////////////////////////////////////////////////////////////////
int float_data_type::get_length() const {
    return 4;
}

void float_data_type::print_repr(std::ostream &stream) const {
    stream << "Float";
}

void float_data_type::format(std::ostream &stream, const void *data) const {
    stream << *((float *) data);
}

////////////////////////////////////////////////////////////////////////////////
// bool_data_type
////////////////////////////////////////////////////////////////////////////////
int bool_data_type::get_length() const {
    return 1;
}

void bool_data_type::print_repr(std::ostream &stream) const {
    stream << "Bool";
}

void bool_data_type::format(std::ostream &stream, const void *data) const {
    stream << (*((char *) data) != 0 ? "true" : "false");
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

int array_data_type::get_array_depth() const {
    return element_type->get_array_depth() + 1;
}

void array_data_type::print_repr(std::ostream &stream) const {
    element_type->print_repr(stream);
    stream << "[" << std::to_string(length) << "]";
}

void array_data_type::format(std::ostream &stream, const void *data) const {
    stream << "[";
    for (int i = 0; i < length; i++) {
        element_type->format(stream, 
            ((char *) data) + i * element_type->get_length());
        if (i != length - 1) {
            stream << ", ";
        }
    }
    stream << "]";
}

int array_data_type::get_array_length() const {
    return length;
}

std::shared_ptr<const data_type> array_data_type::get_element_type() const {
    return element_type;
}

#pragma GCC diagnostic ignored "-Wunused-parameter"
std::shared_ptr<value> 
    array_data_type::get_data_offset(std::shared_ptr<value> index)  const {
    // TODO: Implementation
    return std::shared_ptr<value>{nullptr};
}
#pragma GCC diagnostic pop

////////////////////////////////////////////////////////////////////////////////
// copy_array_data_proxy
////////////////////////////////////////////////////////////////////////////////
copy_array_data_proxy::copy_array_data_proxy(
    std::shared_ptr<const data_type> source_type, int length)
    : array_data_type{source_type, length} { }

bool copy_array_data_proxy::is_proxy_type() const {
    return true;
}

void copy_array_data_proxy::format(std::ostream &stream, const void *data) const {
    stream << "[";
    for (int i = 0; i < get_array_length(); i++) {
        get_element_type()->format(stream, data);
        if (i != get_array_length() - 1) {
            stream << ", ";
        }
    }
    stream << "]";
}

void copy_array_data_proxy::print_repr(std::ostream &stream) const {
    get_element_type()->print_repr(stream);
    stream << "[" << std::to_string(get_array_length()) << " copied from 1]";
}

#pragma GCC diagnostic ignored "-Wunused-parameter"
std::shared_ptr<value> 
    copy_array_data_proxy::get_data_offset(std::shared_ptr<value> index) const {
    // TODO: Implementation
    return std::shared_ptr<value>{nullptr};
}
#pragma GCC diagnostic pop

}
}
