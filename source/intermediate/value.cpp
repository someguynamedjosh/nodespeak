#include <waveguide/intermediate/value.hpp>

#include <cassert>
#include <cstring>
#include <sstream>
#include <string>

#include <waveguide/intermediate/data_type.hpp>

namespace waveguide {
namespace intermediate {

////////////////////////////////////////////////////////////////////////////////
// Com::Value
////////////////////////////////////////////////////////////////////////////////

Value::Value(std::shared_ptr<DataType> type)
    : type{type} {
    if (!type->is_proxy_type()) {
        data = malloc(type->get_length());
    }
}

Value::Value(std::shared_ptr<DataType> type, void *data)
    : type{type}, data{data}, value_known{true} { }

Value::~Value() {
    if (!type->is_proxy_type()) {
        free(data);
    }
}

std::string Value::repr() {
    std::stringstream ss;
    ss << (is_value_known() ? "C" : "V");
    ss << "@" << (void*) this;
    ss << " T=" << type->repr();
    if (is_value_known()) {
        ss << " V=" << type->format(get_real_value().get_data());
    }
    return ss.str();
}

std::shared_ptr<DataType> Value::get_type() {
    return type;
}

void Value::set_type(std::shared_ptr<DataType> new_type) {
    assert(new_type->get_length() == type->get_length());
    assert(new_type->is_proxy_type() == type->is_proxy_type());
    type = new_type;
}

bool Value::is_proxy() {
    return type->is_proxy_type();
}

Value &Value::get_real_value() {
    if (is_proxy()) {
        return static_cast<Value*>(data)->get_real_value();
    } else {
        return *this;
    }
}

bool Value::is_value_known() {
    return value_known;
}

void Value::set_value_known(bool is_known) {
    value_known = is_known;
}

Value Value::create_known_copy() {
    assert(value_known);
    Value tr{type};
    memcpy(tr.get_data(), data, type->get_length());
    tr.set_value_known(true);
    return tr;
}

void *Value::get_data() {
    assert(!is_proxy());
    return data;
}

float *Value::data_as_float() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<FloatDataType>(type));
    return static_cast<float*>(data);
}

int *Value::data_as_int() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<IntDataType>(type));
    return static_cast<int*>(data);
}

bool *Value::data_as_bool() {
    assert(!is_proxy());
    assert(std::dynamic_pointer_cast<BoolDataType>(type));
    return static_cast<bool*>(data);
}

}
}