#pragma once

#include <boost/variant.hpp>
#include <map>
#include <memory>
#include <ostream>
#include <string>
#include <vector>

namespace waveguide {
namespace vague {

class adjective;
class command_lambda;
class scope;
class value_accessor;

typedef std::shared_ptr<adjective> adjective_ptr;
typedef std::shared_ptr<command_lambda> command_lambda_ptr;
typedef std::shared_ptr<scope> scope_ptr;
typedef std::shared_ptr<value_accessor> value_accessor_ptr;
typedef std::shared_ptr<const value_accessor> const_value_accessor_ptr;

struct do_if_aug {
    value_accessor_ptr condition;
    friend std::ostream &operator<<(std::ostream &stream, 
        do_if_aug const&to_print);
};

struct do_if_not_aug {
    value_accessor_ptr condition;
    friend std::ostream &operator<<(std::ostream &stream, 
        do_if_not_aug const&to_print);
};

struct loop_for_aug {
    value_accessor_ptr to_set, iterate_over;
    friend std::ostream &operator<<(std::ostream &stream, 
        loop_for_aug const&to_print);
};

struct loop_range_aug {
    value_accessor_ptr to_set, start, end, step;
    friend std::ostream &operator<<(std::ostream &stream, 
        loop_range_aug const&to_print);
};

std::ostream &operator<<(std::ostream &stream, do_if_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, do_if_not_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, loop_for_aug const&to_print);
std::ostream &operator<<(std::ostream &stream, loop_range_aug const&to_print);

typedef boost::variant<do_if_aug, do_if_not_aug, loop_for_aug, loop_range_aug>
    augmentation;
typedef std::shared_ptr<augmentation> augmentation_ptr;

class adjective {
private:
    typedef const_value_accessor_ptr arg_ptr;
    typedef std::vector<arg_ptr> arg_list;
    arg_list ins, outs;
    std::string name;
public:
    adjective();
    adjective(std::string name);

    std::string const&get_name() const;
    void set_name(std::string name);

    arg_list const&get_inputs() const;
    void add_input(arg_ptr input);
    void clear_inputs();

    arg_list const&get_outputs() const;
    void add_output(arg_ptr output);
    void clear_outputs();
};

class command_lambda {
private:
    typedef std::vector<adjective_ptr> adjective_list;
    adjective_list adjectives;
    scope_ptr scope;
public:
    command_lambda();
    command_lambda(scope_ptr scope);

    scope_ptr get_scope() const;
    void set_scope(scope_ptr scope);

    adjective_list const&get_adjectives() const;
    void add_adjective(adjective_ptr adjective);
    void clear_adjectives();
};

class command {
private:
    typedef const_value_accessor_ptr arg_ptr;
    typedef std::vector<arg_ptr> arg_list;
    arg_list ins, outs;
    std::vector<command_lambda_ptr> lambdas;
    scope_ptr callee{nullptr};
    augmentation_ptr aug{nullptr};

public:
    command();
    command(scope_ptr callee);
    command(scope_ptr callee, augmentation_ptr aug);
    friend std::ostream &operator<<(std::ostream &stream, 
        command const&to_print);

    arg_list const&get_inputs() const;
    void add_input(arg_ptr input);
    void clear_inputs();

    arg_list const&get_outputs() const;
    void add_output(arg_ptr output);
    void clear_outputs();

    std::vector<command_lambda_ptr> const&get_lambdas() const;
    void add_lambda(command_lambda_ptr lambda);
    void clear_lambdas();
    void add_adjective(adjective_ptr adjective);

    const augmentation_ptr get_augmentation() const;

    const scope_ptr get_callee() const;
    void set_callee(scope_ptr callee);
};
std::ostream &operator<<(std::ostream &stream, command const&to_print);

}
}