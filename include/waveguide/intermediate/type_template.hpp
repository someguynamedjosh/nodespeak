#pragma once

#include <memory>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

class data_type;
class value;

class vague_expression { };

class vague_number_expression: public vague_expression {
private:
    int value;
public:
    vague_number_expression(int value);
    int get_value() const;
};

class vague_value_expression: public vague_expression {
private:
    std::string name;
public:
    vague_value_expression(std::string name);
    std::string get_name() const;
};

class vague_known_value_expression: public vague_expression {
private:
    std::shared_ptr<value> real_value;
public:
    vague_known_value_expression(std::shared_ptr<value> real_value);
    std::shared_ptr<value> get_real_value() const;
};

class vague_operation_expression: public vague_expression {
private:
    std::vector<std::shared_ptr<vague_expression>> operands;
protected:
    vague_operation_expression(std::shared_ptr<vague_expression> a);
    vague_operation_expression(std::shared_ptr<vague_expression> a, 
        std::shared_ptr<vague_expression> b);
public:
    std::vector<std::shared_ptr<vague_expression>> const&get_operands() const;
};

class vague_negation_expression: public vague_operation_expression {
public:
    vague_negation_expression(std::shared_ptr<vague_expression> input);
};

class vague_add_expression: public vague_operation_expression {
public:
    vague_add_expression(std::shared_ptr<vague_expression> a, 
        std::shared_ptr<vague_expression> b);
};

class vague_subtract_expression: public vague_operation_expression {
public:
    vague_subtract_expression(std::shared_ptr<vague_expression> a, 
        std::shared_ptr<vague_expression> b);
};

class vague_multiply_expression: public vague_operation_expression {
public:
    vague_multiply_expression(std::shared_ptr<vague_expression> a, 
        std::shared_ptr<vague_expression> b);
};

class vague_divide_expression: public vague_operation_expression {
public:
    vague_divide_expression(std::shared_ptr<vague_expression> a, 
        std::shared_ptr<vague_expression> b);
};



class vague_data_type {
public:
    virtual void print_repr(std::ostream &stream) const = 0;
};

class vague_basic_data_type: public vague_data_type {
private:
    std::string name;
public:
    vague_basic_data_type(std::string name);
    virtual void print_repr(std::ostream &stream) const;
    std::string get_name() const;
};

class vague_known_data_type: public vague_data_type {
private:
    std::shared_ptr<data_type> real_type;
public:
    vague_known_data_type(std::shared_ptr<data_type> real_type);
    virtual void print_repr(std::ostream &stream) const;
    std::shared_ptr<data_type> get_real_type() const;
};

class vague_array_data_type: public vague_data_type {
private:
    std::shared_ptr<vague_data_type> base;
    std::shared_ptr<vague_expression> size;
public:
    vague_array_data_type(std::shared_ptr<vague_data_type> base,
        std::shared_ptr<vague_expression> size);
    virtual void print_repr(std::ostream &stream) const;
    std::shared_ptr<vague_data_type> get_base_type() const;
    std::shared_ptr<vague_expression> get_size() const;
};

}
}