#include <waveguide/intermediate/value.hpp>

#include <waveguide/intermediate/builtins.hpp>
#include <waveguide/intermediate/data_type.hpp>
#include <cassert>
#include <cstring>
#include <iterator>
#include <sstream>
#include <string>

#include "util/aliases.hpp"
#include "util.hpp"

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// Com::value
////////////////////////////////////////////////////////////////////////////////

value::value(const_data_type_ptr type)
    : type{type} {
    if (!type->is_proxy_type()) {
        data = shared_data_block_ptr{new char[type->get_length()]};
    }
}

value::value(const_data_type_ptr type, shared_data_block_ptr data)
    : type{type}, data{data}, value_known{true} {
    value_known = !type->is_proxy_type();
}

value::value(const_data_type_ptr type, data_block_ptr data)
    : type{type}, value_known{true} {
    assert(!type->is_proxy_type());
    value_known = true;
    this->data = shared_data_block_ptr{new char[type->get_length()]};
    for (int i = 0; i < type->get_length(); i++) {
        this->data[i] = data[i];
    }
}

value::value(const_data_type_ptr type, value_ptr target)
    : type{type}, value_known{false} {
    assert(type->is_proxy_type());
    data = std::reinterpret_pointer_cast<char[]>(target);
}

std::shared_ptr<const data_type> value::get_type() const {
    return type;
}

void value::set_type(std::shared_ptr<const data_type> new_type) {
    assert(new_type->get_length() == type->get_length());
    assert(new_type->is_proxy_type() == type->is_proxy_type());
    type = new_type;
}

bool value::is_proxy() const {
    return type->is_proxy_type();
}

value const&value::get_real_value() const {
    if (is_proxy()) {
        return std::reinterpret_pointer_cast<const value>(data)->get_real_value();
    } else {
        return *this;
    }
}

bool value::is_value_known() const {
    return is_proxy() ? get_real_value().is_value_known() : value_known;
}

void value::set_value_known(bool is_known) {
    assert(!is_proxy());
    value_known = is_known;
}

value value::create_known_copy() const {
    assert(value_known);
    value tr{type};
    auto tr_data = tr.get_data();
    for (int i = 0; i < type->get_length(); i++) {
        tr_data[i] = data[i];
    }
    tr.set_value_known(true);
    return tr;
}

const_data_block_ptr value::get_data() const {
    assert(!is_proxy());
    return data.get();
}

float const&value::data_as_float() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const float_data_type>(type));
    return *std::reinterpret_pointer_cast<float>(data);
}

int const&value::data_as_int() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const int_data_type>(type));
    return *std::reinterpret_pointer_cast<int>(data);
}

bool const&value::data_as_bool() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const bool_data_type>(type));
    return *std::reinterpret_pointer_cast<bool>(data);
}

data_block_ptr value::get_data() {
    assert(!is_proxy());
    return data.get();
}

float &value::data_as_float() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const float_data_type>(type));
    return *std::reinterpret_pointer_cast<float>(data);
}

int &value::data_as_int() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const int_data_type>(type));
    return *std::reinterpret_pointer_cast<int>(data);
}

bool &value::data_as_bool() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const bool_data_type>(type));
    return *std::reinterpret_pointer_cast<bool>(data);
}



const_data_block_ptr value_accessor::get_element_ptr() const {
    assert(root_value);
    const_data_block_ptr ptr = root_value.get()->get_data();
    const_data_type_ptr data_type = root_value->get_type();

    for (auto subpart : subparts) {
        ptr = ptr;
    }

    return ptr;
}

value_accessor::value_accessor() { }

value_accessor::value_accessor(const_value_ptr root_value)
    : root_value{root_value} { }

void value_accessor::set_root_value(const_value_ptr root_value) {
    this->root_value = root_value;
}

const_value_ptr value_accessor::get_root_value() const {
    return root_value;
}

void value_accessor::add_subpart(const_value_ptr subpart) {
    subparts.push_back(subpart);
}

std::vector<const_value_ptr> const&value_accessor::get_subparts() const {
    return subparts;
}

bool value_accessor::is_value_known() const {
    if (!root_value->is_value_known()) return false;
    for (auto subpart : subparts) {
        if (!subpart->is_value_known()) return false;
    }
}

const_data_type_ptr value_accessor::get_type() const {

}

const_data_block_ptr value_accessor::get_data() const {
    assert(root_value);
    const_data_block_ptr ptr = root_value.get()->get_data();
    const_data_type_ptr data_type = root_value->get_type();

    for (auto subpart : subparts) {
        assert(subpart->get_type() == blt()->INT);
        ptr = ptr;
    }

    return ptr;
}

float const&value_accessor::data_as_float() const {
    assert(is_value_known());
}

}
}