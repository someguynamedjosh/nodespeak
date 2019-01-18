#include <waveguide/intermediate/type_template.hpp>

#include <ostream>
#include <waveguide/intermediate/data_type.hpp>
#include <waveguide/intermediate/value.hpp>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

vague_number_expression::vague_number_expression(int value)
    : value{value} { }

void vague_number_expression::print_repr(std::ostream &stream) const {
    stream << value;
}

void vague_number_expression::collect_new_vars(std::vector<std::string> &list) 
    const { }

bool vague_number_expression::is_constant() const {
    return true;
}

int vague_number_expression::do_algebra(possible_value_table &values,
    int final_value) const {
    return value;
}

int vague_number_expression::resolve_value(
    resolved_value_table const&value_table) const {
    return value;
}

int vague_number_expression::get_value() const {
    return value;
}

vague_value_expression::vague_value_expression(std::string name)
    : name{name} { }

void vague_value_expression::print_repr(std::ostream &stream) const {
    stream << name << "?";
}

void vague_value_expression::collect_new_vars(std::vector<std::string> &list) 
    const {
    list.push_back(name);
}

bool vague_value_expression::is_constant() const {
    return false;
}

int vague_value_expression::do_algebra(possible_value_table &values,
    int final_value) const {
    values[name].push_back(final_value);
    return final_value;
}

int vague_value_expression::resolve_value(
    resolved_value_table const&value_table) const {
    return value_table.find(name)->second;
}

std::string vague_value_expression::get_name() const {
    return name;
}

vague_known_value_expression::vague_known_value_expression(value_ptr real_value)
    : real_value{real_value} { }

void vague_known_value_expression::print_repr(std::ostream &stream) const {
    stream << real_value;
}

void vague_known_value_expression::collect_new_vars(
    std::vector<std::string> &list) const { }

bool vague_known_value_expression::is_constant() const {
    assert(real_value->is_value_known());
    return true;
}

int vague_known_value_expression::do_algebra(possible_value_table &values,
    int final_value) const {
    assert(real_value->is_value_known());
    return *real_value->data_as_int();
}

int vague_known_value_expression::resolve_value(
    resolved_value_table const&value_table) const {
    assert(real_value->is_value_known());
    return *real_value->data_as_int();
}

value_ptr vague_known_value_expression::get_real_value() const {
    return real_value;
}

vague_operation_expression::vague_operation_expression(vague_expression_ptr a)
    : operands{a} { }

vague_operation_expression::vague_operation_expression(vague_expression_ptr a, 
    vague_expression_ptr b)
    : operands{a, b} { }

std::vector<vague_expression_ptr> 
    const&vague_operation_expression::get_operands() const {
    return operands;
}

void vague_operation_expression::collect_new_vars(
    std::vector<std::string> &list) const {
    for (auto operand : operands) {
        operand->collect_new_vars(list);
    }
}

bool vague_operation_expression::is_constant() const {
    bool result = true;
    for (auto operand : operands) {
        result &= operand->is_constant();
    }
    return result;
}

vague_negation_expression::vague_negation_expression(vague_expression_ptr input)
    : vague_operation_expression{input} { }

void vague_negation_expression::print_repr(std::ostream &stream) const {
    auto const&ops = get_operands();
    stream << "(-";
    ops[0]->print_repr(stream);
    stream << ")";
}

int vague_negation_expression::do_algebra(possible_value_table &table,
    int final_value) const {
    get_operands()[0]->do_algebra(table, -final_value);
    return final_value;
}

int vague_negation_expression::resolve_value(
    resolved_value_table const&value_table) const {
    return get_operands()[0]->resolve_value(value_table);
}

vague_add_expression::vague_add_expression(vague_expression_ptr a, 
    vague_expression_ptr b)
    : vague_operation_expression{a, b} { }

void vague_add_expression::print_repr(std::ostream &stream) const {
    auto const&ops = get_operands();
    stream << "(";
    ops[0]->print_repr(stream);
    stream << " + ";
    ops[1]->print_repr(stream);
    stream << ")";
}

int vague_add_expression::do_algebra(possible_value_table &table,
    int final_value) const {
    auto a = get_operands()[0], b = get_operands()[1];
    if (a->is_constant()) {
        final_value -= a->do_algebra(table, final_value);
        b->do_algebra(table, final_value);
    } else if (b->is_constant()) {
        final_value -= b->do_algebra(table, final_value);
        a->do_algebra(table, final_value);
    } else {
        return 0;
    }
    return final_value;
}

int vague_add_expression::resolve_value(
    resolved_value_table const&value_table) const {
    int a = get_operands()[0]->resolve_value(value_table);
    int b = get_operands()[1]->resolve_value(value_table);
    return a + b;
}

vague_subtract_expression::vague_subtract_expression(vague_expression_ptr a, 
    vague_expression_ptr b)
    : vague_operation_expression{a, b} { }

void vague_subtract_expression::print_repr(std::ostream &stream) const {
    auto const&ops = get_operands();
    stream << "(";
    ops[0]->print_repr(stream);
    stream << " - ";
    ops[1]->print_repr(stream);
    stream << ")";
}

int vague_subtract_expression::do_algebra(possible_value_table &table,
    int final_value) const {
    auto a = get_operands()[0], b = get_operands()[1];
    if (a->is_constant()) {
        final_value = a->do_algebra(table, final_value);
        b->do_algebra(table, final_value);
    } else if (b->is_constant()) {
        final_value += b->do_algebra(table, final_value);
        a->do_algebra(table, final_value);
    } else {
        return 0;
    }
    return final_value;
}

int vague_subtract_expression::resolve_value(
    resolved_value_table const&value_table) const {
    int a = get_operands()[0]->resolve_value(value_table);
    int b = get_operands()[1]->resolve_value(value_table);
    return a - b;
}

vague_multiply_expression::vague_multiply_expression(vague_expression_ptr a,
    vague_expression_ptr b)
    : vague_operation_expression{a, b} { }

void vague_multiply_expression::print_repr(std::ostream &stream) const {
    auto const&ops = get_operands();
    stream << "(";
    ops[0]->print_repr(stream);
    stream << " * ";
    ops[1]->print_repr(stream);
    stream << ")";
}

int vague_multiply_expression::do_algebra(possible_value_table &table,
    int final_value) const {
    auto a = get_operands()[0], b = get_operands()[1];
    if (a->is_constant()) {
        int divisor = a->do_algebra(table, final_value);
        if (final_value % divisor != 0) {
            return 0;
        }
        b->do_algebra(table, final_value / divisor);
    } else if (b->is_constant()) {
        int divisor = b->do_algebra(table, final_value);
        if (final_value % divisor != 0) {
            return 0;
        }
        a->do_algebra(table, final_value / divisor);
    } else {
        return 0;
    }
    return final_value;
}

int vague_multiply_expression::resolve_value(
    resolved_value_table const&value_table) const {
    int a = get_operands()[0]->resolve_value(value_table);
    int b = get_operands()[1]->resolve_value(value_table);
    return a * b;
}

vague_divide_expression::vague_divide_expression(vague_expression_ptr a, 
    vague_expression_ptr b)
    : vague_operation_expression{a, b} { }

void vague_divide_expression::print_repr(std::ostream &stream) const {
    auto const&ops = get_operands();
    stream << "(";
    ops[0]->print_repr(stream);
    stream << " / ";
    ops[1]->print_repr(stream);
    stream << ")";
}

int vague_divide_expression::do_algebra(possible_value_table &table,
    int final_value) const {
    auto a = get_operands()[0], b = get_operands()[1];
    if (a->is_constant()) {
        int top = a->do_algebra(table, final_value);
        if (top % final_value != 0) {
            return 0;
        }
        b->do_algebra(table, top / final_value);
    } else if (b->is_constant()) {
        int divisor = b->do_algebra(table, final_value);
        a->do_algebra(table, final_value * divisor);
    } else {
        return 0;
    }
    return final_value;
}

int vague_divide_expression::resolve_value(
    resolved_value_table const&value_table) const {
    int a = get_operands()[0]->resolve_value(value_table);
    int b = get_operands()[1]->resolve_value(value_table);
    return a / b;
}



vague_basic_data_type::vague_basic_data_type(std::string name)
    : name{name} { }

void vague_basic_data_type::print_repr(std::ostream &stream) const {
    stream << name << "?";
}

void vague_basic_data_type::collect_new_vars(std::vector<std::string> &list)
    const { }

void vague_basic_data_type::collect_new_types(std::vector<std::string> &list)
    const {
    list.push_back(name);
}

bool vague_basic_data_type::fill_tables(possible_value_table &value_table,
    data_type_table &type_table, const_data_type_ptr real_type) const {
    type_table[name].push_back(real_type);
    return true;
}

const_data_type_ptr vague_basic_data_type::resolve_type(resolved_value_table 
    const&value_table, resolved_data_type_table const&type_table)
    const {
    return type_table.find(name)->second;
}

std::string vague_basic_data_type::get_name() const {
    return name;
}

vague_known_data_type::vague_known_data_type(const_data_type_ptr real_type)
    : real_type{real_type} { }

void vague_known_data_type::print_repr(std::ostream &stream) const {
    real_type->print_repr(stream);
}

void vague_known_data_type::collect_new_vars(std::vector<std::string> &list)
    const { }

void vague_known_data_type::collect_new_types(std::vector<std::string> &list)
    const { }

bool vague_known_data_type::fill_tables(possible_value_table &value_table,
    data_type_table &type_table, const_data_type_ptr real_type) const {
    // TODO: Check for equality between real_type and this->real_type.
    return true;
}

const_data_type_ptr vague_known_data_type::resolve_type(resolved_value_table 
    const&value_table, resolved_data_type_table const&type_table)
    const {
    return real_type;
}

const_data_type_ptr vague_known_data_type::get_real_type() const {
    return real_type;
}

vague_array_data_type::vague_array_data_type(vague_const_data_type_ptr base, 
    vague_expression_ptr size)
    : base(base), size(size) { }

void vague_array_data_type::print_repr(std::ostream &stream) const {
    stream << "[";
    size->print_repr(stream);
    stream << "]";
    base->print_repr(stream);
}

void vague_array_data_type::collect_new_vars(std::vector<std::string> &list)
    const {
    size->collect_new_vars(list);
}

void vague_array_data_type::collect_new_types(std::vector<std::string> &list)
    const {
    base->collect_new_types(list);
}

bool vague_array_data_type::fill_tables(possible_value_table &value_table,
    data_type_table &type_table, const_data_type_ptr real_type) const {
    auto array_type = std::dynamic_pointer_cast<const array_data_type>(
        real_type
    );
    if (array_type) {
        int asize = size->do_algebra(value_table, array_type->get_array_length());
        if (asize != array_type->get_array_length()) return false;
        base->fill_tables(value_table, type_table, std::static_pointer_cast
            <const data_type>(array_type->get_element_type())
        );
    } else {
        size->do_algebra(value_table, 1);
        base->fill_tables(value_table, type_table, real_type);
    }
    return true;
}

const_data_type_ptr vague_array_data_type::resolve_type(resolved_value_table 
    const&value_table, resolved_data_type_table const&type_table)
    const {
    auto sub_type = base->resolve_type(value_table, type_table);
    auto real_size = size->resolve_value(value_table);
    return std::make_shared<intr::array_data_type>(sub_type, real_size);
}

vague_const_data_type_ptr vague_array_data_type::get_base_type() const {
    return base;
}

vague_expression_ptr vague_array_data_type::get_size() const {
    return size;
}

}
}