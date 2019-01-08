#pragma once

#include <map>
#include <memory>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

class data_type;
class vague_data_type;
class vague_expression;
class value;

typedef std::shared_ptr<data_type> data_type_ptr;
typedef std::shared_ptr<vague_data_type> vague_data_type_ptr;
typedef std::shared_ptr<vague_expression> vague_expression_ptr;
typedef std::shared_ptr<value> value_ptr;

class vague_expression {
public:
    virtual void print_repr(std::ostream &stream) const = 0;
    virtual void collect_new_vars(std::vector<std::string> &list) const = 0;
    virtual bool is_constant() = 0;
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value) = 0;
};

class vague_number_expression: public vague_expression {
private:
    int value;
public:
    vague_number_expression(int value);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual bool is_constant();
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value);
    int get_value() const;
};

class vague_value_expression: public vague_expression {
private:
    std::string name;
public:
    vague_value_expression(std::string name);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual bool is_constant();
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value);
    std::string get_name() const;
};

class vague_known_value_expression: public vague_expression {
private:
    value_ptr real_value;
public:
    vague_known_value_expression(value_ptr real_value);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual bool is_constant();
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value);
    value_ptr get_real_value() const;
};

class vague_operation_expression: public vague_expression {
private:
    std::vector<vague_expression_ptr> operands;
protected:
    vague_operation_expression(vague_expression_ptr a);
    vague_operation_expression(vague_expression_ptr a, vague_expression_ptr b);
public:
    std::vector<vague_expression_ptr> const&get_operands() const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual bool is_constant();
};

class vague_negation_expression: public vague_operation_expression {
public:
    vague_negation_expression(vague_expression_ptr input);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value);
};

class vague_add_expression: public vague_operation_expression {
public:
    vague_add_expression(vague_expression_ptr a, vague_expression_ptr b);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value);
};

class vague_subtract_expression: public vague_operation_expression {
public:
    vague_subtract_expression(vague_expression_ptr a, vague_expression_ptr b);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value);
};

class vague_multiply_expression: public vague_operation_expression {
public:
    vague_multiply_expression(vague_expression_ptr a, vague_expression_ptr b);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value);
};

class vague_divide_expression: public vague_operation_expression {
public:
    vague_divide_expression(vague_expression_ptr a, vague_expression_ptr b);
    virtual void print_repr(std::ostream &stream) const;
    virtual int do_algebra(std::map<std::string, std::vector<int>> &table,
        int final_value);
};



class vague_data_type {
public:
    virtual void print_repr(std::ostream &stream) const = 0;
    virtual void collect_new_vars(std::vector<std::string> &list) const = 0;
    virtual void collect_new_types(std::vector<std::string> &list) const = 0;
};

class vague_basic_data_type: public vague_data_type {
private:
    std::string name;
public:
    vague_basic_data_type(std::string name);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual void collect_new_types(std::vector<std::string> &list) const;
    std::string get_name() const;
};

class vague_known_data_type: public vague_data_type {
private:
    data_type_ptr real_type;
public:
    vague_known_data_type(data_type_ptr real_type);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual void collect_new_types(std::vector<std::string> &list) const;
    data_type_ptr get_real_type() const;
};

class vague_array_data_type: public vague_data_type {
private:
    vague_data_type_ptr base;
    vague_expression_ptr size;
public:
    vague_array_data_type(vague_data_type_ptr base,
        vague_expression_ptr size);
    virtual void print_repr(std::ostream &stream) const;
    virtual void collect_new_vars(std::vector<std::string> &list) const;
    virtual void collect_new_types(std::vector<std::string> &list) const;
    vague_data_type_ptr get_base_type() const;
    vague_expression_ptr get_size() const;
};

}
}