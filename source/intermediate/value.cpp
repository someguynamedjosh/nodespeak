#include <waveguide/intermediate/value.hpp>

#include <cassert>
#include <cstring>
#include <sstream>
#include <string>

#include <waveguide/intermediate/data_type.hpp>

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// Com::value
////////////////////////////////////////////////////////////////////////////////

value::value(std::shared_ptr<const data_type> type)
    : type{type} {
    if (!type->is_proxy_type()) {
        data = malloc(type->get_length());
    }
}

value::value(std::shared_ptr<const data_type> type, void *data)
    : type{type}, data{data}, value_known{true} { }

value::~value() {
    if (!type->is_proxy_type()) {
        free(data);
    }
}

const std::string value::repr() const {
    std::stringstream ss;
    ss << (is_value_known() ? "C" : "V");
    ss << "@" << (void*) this;
    ss << " T=" << type->repr();
    if (is_value_known()) {
        ss << " V=" << type->format(get_real_value().get_data());
    }
    return ss.str();
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
        return static_cast<const value*>(data)->get_real_value();
    } else {
        return *this;
    }
}

bool value::is_value_known() const {
    return value_known;
}

void value::set_value_known(bool is_known) {
    value_known = is_known;
}

value value::create_known_copy() const {
    assert(value_known);
    value tr{type};
    memcpy(tr.get_data(), data, type->get_length());
    tr.set_value_known(true);
    return tr;
}

const void *value::get_data() const {
    assert(!is_proxy());
    return data;
}

const float *value::data_as_float() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<float_data_type>(type));
    return static_cast<float*>(data);
}

const int *value::data_as_int() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<int_data_type>(type));
    return static_cast<int*>(data);
}

const bool *value::data_as_bool() const {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<bool_data_type>(type));
    return static_cast<bool*>(data);
}

void *value::get_data() {
    assert(!is_proxy());
    return data;
}

float *value::data_as_float() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<float_data_type>(type));
    return static_cast<float*>(data);
}

int *value::data_as_int() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<int_data_type>(type));
    return static_cast<int*>(data);
}

bool *value::data_as_bool() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<bool_data_type>(type));
    return static_cast<bool*>(data);
}

}
}