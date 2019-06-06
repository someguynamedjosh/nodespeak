#pragma once

#include <map>
#include <memory>
#include <ostream>
#include <string>
#include <vector>

namespace waveguide {
namespace vague {

class data_type;
class template_data_type;
class template_expression;

typedef std::shared_ptr<data_type> data_type_ptr;
typedef std::shared_ptr<const data_type> const_data_type_ptr;
typedef std::shared_ptr<template_data_type> template_data_type_ptr;
typedef std::shared_ptr<template_expression> template_expression_ptr;
typedef std::map<std::string, std::vector<const_data_type_ptr>> data_type_table;
typedef std::map<std::string, const_data_type_ptr> resolved_data_type_table;
typedef std::map<std::string, std::vector<int>> possible_value_table;
typedef std::map<std::string, int> resolved_value_table;

class template_expression {
public:
    virtual void print_repr(std::ostream &stream) const = 0;
    virtual bool is_constant() const = 0;
    virtual void collect_new_vars(std::vector<std::string> &list) const = 0;
    virtual int do_algebra(possible_value_table &table, int final_value) const
        = 0;
    virtual int resolve_value(resolved_value_table const&value_table) const = 0;
};

class template_number_expression: public template_expression {
private:
    int value;
public:
    template_number_expression(int value);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual bool is_constant() const;
    virtual int do_algebra(possible_value_table &table, int final_value) const;
    virtual int resolve_value(resolved_value_table const&value_table) const;
    int get_value() const;
};

class template_value_expression: public template_expression {
private:
    std::string name;
public:
    template_value_expression(std::string name);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual bool is_constant() const;
    virtual int do_algebra(possible_value_table &table, int final_value) const;
    virtual int resolve_value(resolved_value_table const&value_table) const;
    std::string get_name() const;
};

class template_operation_expression: public template_expression {
private:
    std::vector<template_expression_ptr> operands;
protected:
    template_operation_expression(template_expression_ptr a);
    template_operation_expression(template_expression_ptr a, template_expression_ptr b);
public:
    std::vector<template_expression_ptr> const&get_operands() const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual bool is_constant() const;
};

class template_negation_expression: public template_operation_expression {
public:
    template_negation_expression(template_expression_ptr input);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(possible_value_table &table, int final_value) const;
    virtual int resolve_value(resolved_value_table const&value_table) const;
};

class template_add_expression: public template_operation_expression {
public:
    template_add_expression(template_expression_ptr a, template_expression_ptr b);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(possible_value_table &table, int final_value) const;
    virtual int resolve_value(resolved_value_table const&value_table) const;
};

class template_subtract_expression: public template_operation_expression {
public:
    template_subtract_expression(template_expression_ptr a, template_expression_ptr b);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(possible_value_table &table, int final_value) const;
    virtual int resolve_value(resolved_value_table const&value_table) const;
};

class template_multiply_expression: public template_operation_expression {
public:
    template_multiply_expression(template_expression_ptr a, template_expression_ptr b);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(possible_value_table &table, int final_value) const;
    virtual int resolve_value(resolved_value_table const&value_table) const;
};

class template_divide_expression: public template_operation_expression {
public:
    template_divide_expression(template_expression_ptr a, template_expression_ptr b);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(possible_value_table &table, int final_value) const;
    virtual int resolve_value(resolved_value_table const&value_table) const;
};



class template_data_type {
public:
    virtual void print_repr(std::ostream &stream) const = 0;
    virtual void collect_new_vars(std::vector<std::string> &list) const = 0;
    virtual void collect_new_types(std::vector<std::string> &list) const = 0;
    virtual bool fill_tables(possible_value_table &value_table, 
        data_type_table &type_table, const_data_type_ptr real_type) const = 0;
    virtual const_data_type_ptr resolve_type(resolved_value_table 
        const&value_table, resolved_data_type_table const&type_table) const = 0;
};

class template_wildcard_data_type: public template_data_type {
private:
    std::string name;
public:
    template_wildcard_data_type(std::string name);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual void collect_new_types(std::vector<std::string> &list) const;
    virtual bool fill_tables(possible_value_table &value_table, 
        data_type_table &type_table, const_data_type_ptr real_type) const;
    virtual const_data_type_ptr resolve_type(resolved_value_table 
        const&value_table, resolved_data_type_table const&type_table) const;
    std::string get_name() const;
};

class template_named_data_type: public template_data_type {
private:
    std::string type_name;
public:
    template_named_data_type(std::string type_name);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual void collect_new_types(std::vector<std::string> &list) const;
    virtual bool fill_tables(possible_value_table &value_table, 
        data_type_table &type_table, const_data_type_ptr real_type) const;
    virtual const_data_type_ptr resolve_type(resolved_value_table 
        const&value_table, resolved_data_type_table const&type_table) const;
    std::string get_name() const;
};

class template_array_data_type: public template_data_type {
private:
    template_data_type_ptr base;
    template_expression_ptr size;
public:
    template_array_data_type(template_data_type_ptr base,
        template_expression_ptr size);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual void collect_new_types(std::vector<std::string> &list) const;
    virtual bool fill_tables(possible_value_table &value_table, 
        data_type_table &type_table, const_data_type_ptr real_type) const;
    virtual const_data_type_ptr resolve_type(resolved_value_table 
        const&value_table, resolved_data_type_table const&type_table) const;
    template_data_type_ptr get_base_type() const;
    template_expression_ptr get_size() const;
};

}
}