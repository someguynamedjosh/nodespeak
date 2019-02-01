#include <waveguide/intermediate/value.hpp>

#include <waveguide/intermediate/data_type.hpp>
#include <cassert>
#include <cstring>
#include <iterator>
#include <sstream>
#include <string>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// Com::value
////////////////////////////////////////////////////////////////////////////////

value::value(std::shared_ptr<const data_type> type)
    : type{type} {
    if (!type->is_proxy_type()) {
        data = data_block_ptr{new char[type->get_length()]};
    }
}

value::value(std::shared_ptr<const data_type> type, data_block_ptr data)
    : type{type}, data{data}, value_known{true} {
    value_known = !type->is_proxy_type();
}

value::value(std::shared_ptr<const data_type> type, value_ptr target)
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
    auto tr_data = tr.get_data().get();
    for (int i = 0; i < type->get_length(); i++) {
        tr_data[i] = data[i];
    }
    tr.set_value_known(true);
    return tr;
}

const data_block_ptr value::get_data() const {
    assert(!is_proxy());
    return data;
}

const std::shared_ptr<float> value::data_as_float() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const float_data_type>(type));
    return std::reinterpret_pointer_cast<float>(data);
}

const std::shared_ptr<int> value::data_as_int() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const int_data_type>(type));
    return std::reinterpret_pointer_cast<int>(data);
}

const std::shared_ptr<bool> value::data_as_bool() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const bool_data_type>(type));
    return std::reinterpret_pointer_cast<bool>(data);
}

std::shared_ptr<char[]> value::get_data() {
    assert(!is_proxy());
    return data;
}

std::shared_ptr<float> value::data_as_float() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const float_data_type>(type));
    return std::reinterpret_pointer_cast<float>(data);
}

std::shared_ptr<int> value::data_as_int() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const int_data_type>(type));
    return std::reinterpret_pointer_cast<int>(data);
}

std::shared_ptr<bool> value::data_as_bool() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<const bool_data_type>(type));
    return std::reinterpret_pointer_cast<bool>(data);
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

}
}