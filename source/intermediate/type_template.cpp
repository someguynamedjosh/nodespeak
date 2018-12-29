#include <waveguide/intermediate/type_template.hpp>

#include <ostream>

#include "util/aliases.hpp"

namespace waveguide {
namespace intermediate {

vague_number_expression::vague_number_expression(int value)
    : value{value} { }

int vague_number_expression::get_value() const {
    return value;
}

vague_value_expression::vague_value_expression(std::string name)
    : name(name) { }

std::string vague_value_expression::get_name() const {
    return name;
}

vague_operation_expression::vague_operation_expression(SP<vague_expression> a)
    : operands{a} { }

vague_operation_expression::vague_operation_expression(SP<vague_expression> a, 
    SP<vague_expression> b)
    : operands{a, b} { }

std::vector<SP<vague_expression>> 
    const&vague_operation_expression::get_operands() const {
    return operands;
}

vague_negation_expression::vague_negation_expression(SP<vague_expression> input)
    : vague_operation_expression{input} { }

vague_add_expression::vague_add_expression(SP<vague_expression> a, 
    SP<vague_expression> b)
    : vague_operation_expression{a, b} { }

vague_subtract_expression::vague_subtract_expression(SP<vague_expression> a, 
    SP<vague_expression> b)
    : vague_operation_expression{a, b} { }

vague_multiply_expression::vague_multiply_expression(SP<vague_expression> a,
    SP<vague_expression> b)
    : vague_operation_expression{a, b} { }

vague_divide_expression::vague_divide_expression(SP<vague_expression> a, 
    SP<vague_expression> b)
    : vague_operation_expression{a, b} { }



vague_basic_data_type::vague_basic_data_type(std::string name)
    : name{name} { }

std::string vague_basic_data_type::get_name() const {
    return name;
}

void vague_basic_data_type::print_repr(std::ostream &stream) const {
    stream << name;
}

vague_array_data_type::vague_array_data_type( SP<vague_data_type> base, 
    SP<vague_expression> size)
    : base(base), size(size) { }

void vague_array_data_type::print_repr(std::ostream &stream) const {
    base->print_repr(stream);
    stream << "[]";
    // TODO: Print array size.
}

SP<vague_data_type> vague_array_data_type::get_base_type() const {
    return base;
}

SP<vague_expression> vague_array_data_type::get_size() const {
    return size;
}

}
}