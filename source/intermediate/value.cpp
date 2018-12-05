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

value::value(std::shared_ptr<data_type> type)
    : type{type} {
    if (!type->is_proxy_type()) {
        data = malloc(type->get_length());
    }
}

value::value(std::shared_ptr<data_type> type, void *data)
    : type{type}, data{data}, value_known{true} { }

value::~value() {
    if (!type->is_proxy_type()) {
        free(data);
    }
}

std::string value::repr() {
    std::stringstream ss;
    ss << (is_value_known() ? "C" : "V");
    ss << "@" << (void*) this;
    ss << " T=" << type->repr();
    if (is_value_known()) {
        ss << " V=" << type->format(get_real_value().get_data());
    }
    return ss.str();
}

std::shared_ptr<data_type> value::get_type() {
    return type;
}

void value::set_type(std::shared_ptr<data_type> new_type) {
    assert(new_type->get_length() == type->get_length());
    assert(new_type->is_proxy_type() == type->is_proxy_type());
    type = new_type;
}

bool value::is_proxy() {
    return type->is_proxy_type();
}

value &value::get_real_value() {
    if (is_proxy()) {
        return static_cast<value*>(data)->get_real_value();
    } else {
        return *this;
    }
}

bool value::is_value_known() {
    return value_known;
}

void value::set_value_known(bool is_known) {
    value_known = is_known;
}

value value::create_known_copy() {
    assert(value_known);
    value tr{type};
    memcpy(tr.get_data(), data, type->get_length());
    tr.set_value_known(true);
    return tr;
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