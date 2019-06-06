#include <waveguide/_shared/value.hpp>

// Otherwise there will be undefined references to all the value methods.
#include <waveguide/resolved/value.hpp>
#include <waveguide/vague/value.hpp>

#include <cassert>
#include <cstring>
#include <iterator>
#include <sstream>
#include <string>

namespace waveguide {
namespace _shared {

////////////////////////////////////////////////////////////////////////////////
// Com::value
////////////////////////////////////////////////////////////////////////////////

template<typename Metatype>
value<Metatype>::value(const_metatype_ptr type)
    : type{type} {
    if (!type->is_proxy_type()) {
        data = shared_data_block_ptr{new char[type->get_length()]};
    }
}

template<typename Metatype>
value<Metatype>::value(const_metatype_ptr type, shared_data_block_ptr data)
    : type{type}, data{data}, value_known{true} {
    value_known = !type->is_proxy_type();
}

template<typename Metatype>
value<Metatype>::value(const_metatype_ptr type, data_block_ptr data)
    : type{type}, value_known{true} {
    assert(!type->is_proxy_type());
    value_known = true;
    this->data = shared_data_block_ptr{new char[type->get_length()]};
    for (int i = 0; i < type->get_length(); i++) {
        this->data[i] = data[i];
    }
}

template<typename Metatype>
value<Metatype>::value(const_metatype_ptr type, value_ptr target)
    : type{type}, value_known{false} {
    assert(type->is_proxy_type());
    data = std::reinterpret_pointer_cast<char[]>(target);
}

template<typename Metatype>
void value<Metatype>::set_debug_label(std::string label) {
    debug_label = label;
}

template<typename Metatype>
std::string value<Metatype>::get_debug_label() const {
    return debug_label;
}

template<typename Metatype>
typename value<Metatype>::const_metatype_ptr value<Metatype>::get_type() const {
    return type;
}

template<typename Metatype>
void value<Metatype>::set_type(const_metatype_ptr new_type) {
    // TODO: Genericize?
    // if (type != blt()->DEDUCE_LATER) {
    //     assert(new_type->get_length() == type->get_length());
    //     assert(new_type->is_proxy_type() == type->is_proxy_type());
    // }
    type = new_type;
}

template<typename Metatype>
bool value<Metatype>::is_proxy() const {
    return type->is_proxy_type();
}

template<typename Metatype>
value<Metatype> const&value<Metatype>::get_real_value() const {
    if (is_proxy()) {
        return std::reinterpret_pointer_cast<const value>(data)->get_real_value();
    } else {
        return *this;
    }
}

template<typename Metatype>
bool value<Metatype>::is_value_known() const {
    return is_proxy() ? get_real_value().is_value_known() : value_known;
}

template<typename Metatype>
void value<Metatype>::set_value_known(bool is_known) {
    assert(!is_proxy());
    value_known = is_known;
}

template<typename Metatype>
value<Metatype> value<Metatype>::create_known_copy() const {
    assert(value_known);
    value tr{type};
    auto tr_data = tr.get_data();
    for (int i = 0; i < type->get_length(); i++) {
        tr_data[i] = data[i];
    }
    tr.set_value_known(true);
    return tr;
}

template<typename Metatype>
const_data_block_ptr value<Metatype>::get_data() const {
    assert(!is_proxy());
    return data.get();
}

template<typename Metatype>
float const&value<Metatype>::data_as_float() const {
    assert(!is_proxy());
    // TODO: Implement type checks in child classes?
    // assert(std::dynamic_pointer_cast<const float_data_type>(type));
    return *std::reinterpret_pointer_cast<float>(data);
}

template<typename Metatype>
int const&value<Metatype>::data_as_int() const {
    assert(!is_proxy());
    // assert(std::dynamic_pointer_cast<const int_data_type>(type));
    return *std::reinterpret_pointer_cast<int>(data);
}

template<typename Metatype>
bool const&value<Metatype>::data_as_bool() const {
    assert(!is_proxy());
    // assert(std::dynamic_pointer_cast<const bool_data_type>(type));
    return *std::reinterpret_pointer_cast<bool>(data);
}

template<typename Metatype>
data_block_ptr value<Metatype>::get_data() {
    assert(!is_proxy());
    return data.get();
}

template<typename Metatype>
float &value<Metatype>::data_as_float() {
    assert(!is_proxy());
    // assert(std::dynamic_pointer_cast<const float_data_type>(type));
    return *std::reinterpret_pointer_cast<float>(data);
}

template<typename Metatype>
int &value<Metatype>::data_as_int() {
    assert(!is_proxy());
    // assert(std::dynamic_pointer_cast<const int_data_type>(type));
    return *std::reinterpret_pointer_cast<int>(data);
}

template<typename Metatype>
bool &value<Metatype>::data_as_bool() {
    assert(!is_proxy());
    //assert(std::dynamic_pointer_cast<const bool_data_type>(type));
    return *std::reinterpret_pointer_cast<bool>(data);
}



template<typename ValueType>
value_accessor<ValueType>::value_accessor() { }

template<typename ValueType>
value_accessor<ValueType>::value_accessor(value_ptr root_value)
    : root_value{root_value} { }

template<typename ValueType>
std::string value_accessor<ValueType>::get_debug_label() const {
    std::string output = root_value->get_debug_label();
    for (auto subpart : subparts) {
        output += '[';
        output += subpart->get_debug_label();
        output += ']';
    }
    return output;
}

template<typename ValueType>
void value_accessor<ValueType>::set_root_value(value_ptr root_value) {
    this->root_value = root_value;
}

template<typename ValueType>
typename value_accessor<ValueType>::value_ptr 
value_accessor<ValueType>::get_root_value() const {
    return root_value;
}

template<typename ValueType>
void value_accessor<ValueType>::add_subpart(const_value_accessor_ptr subpart) {
    subparts.push_back(subpart);
}

template<typename ValueType>
std::vector<typename value_accessor<ValueType>::const_value_accessor_ptr> const&
value_accessor<ValueType>::get_subparts() const {
    return subparts;
}

template<typename ValueType>
bool value_accessor<ValueType>::is_value_known() const {
    if (!root_value->is_value_known()) return false;
    for (auto subpart : subparts) {
        if (!subpart->is_value_known()) return false;
    }
    return true;
}

template<typename ValueType>
float const&value_accessor<ValueType>::data_as_float() const {
    return *get_data();
}

template<typename ValueType>
int const&value_accessor<ValueType>::data_as_int() const {
    return *get_data();
}

template<typename ValueType>
bool const&value_accessor<ValueType>::data_as_bool() const {
    return *get_data();
}

template<typename ValueType>
float &value_accessor<ValueType>::data_as_float()  {
    return *(float*)get_data();
}

template<typename ValueType>
int &value_accessor<ValueType>::data_as_int()  {
    return *(int*)get_data();
}

template<typename ValueType>
bool &value_accessor<ValueType>::data_as_bool()  {
    return *(bool*)get_data();
}

}
}