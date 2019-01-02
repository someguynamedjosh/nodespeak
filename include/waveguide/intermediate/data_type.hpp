#pragma once

#include <memory>
#include <string>

namespace waveguide {
namespace intermediate {

class scope;
class vague_data_type;
class value;

class data_type {
public:
    data_type();
    virtual int get_length() const = 0;
    virtual std::shared_ptr<const data_type> get_base_type() const;
    virtual int get_array_depth() const;
    virtual bool is_proxy_type() const;
    virtual void print_repr(std::ostream &stream) const = 0;
    virtual void format(std::ostream &stream, const char *data) const = 0;
};

class abstract_data_type: public data_type {
private:
    std::string label;
public:
    abstract_data_type(std::string label);
    virtual int get_length() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, const char *data) const;
};

class unresolved_vague_type: public data_type {
private:
    std::shared_ptr<vague_data_type> unresolved;
public:
    unresolved_vague_type(std::shared_ptr<vague_data_type> unresolved);
    std::shared_ptr<vague_data_type> get_vague_type() const;
    virtual int get_length() const;
    virtual bool is_proxy_type() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, const char *data) const;
};

class int_data_type: public data_type {
public:
    virtual int get_length() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, const char *data) const;
};

class float_data_type: public data_type {
public:
    virtual int get_length() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, const char *data) const;
};

class bool_data_type: public data_type {
public:
    virtual int get_length() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, const char *data) const;
};

class array_data_type: public data_type {
private:
    std::shared_ptr<const data_type> element_type;
    int length;
public:
    array_data_type(std::shared_ptr<const data_type> element_type, int length);
    virtual int get_length() const;
    virtual std::shared_ptr<const data_type> get_base_type() const;
    virtual int get_array_depth() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, const char *data) const;

    virtual int get_array_length() const;
    std::shared_ptr<const data_type> get_element_type() const;
    virtual std::shared_ptr<value> 
        get_data_offset(std::shared_ptr<value> index) const;
};

class copy_array_data_proxy: public array_data_type {
public:
    copy_array_data_proxy(std::shared_ptr<const data_type> source_type, 
        int length);
    virtual bool is_proxy_type() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, const char *data) const;

    virtual std::shared_ptr<value> 
        get_data_offset(std::shared_ptr<value> index) const;
};

}
}
