#pragma once

#include <waveguide/resolved/value.hpp>
#include <memory>
#include <string>

namespace waveguide {
namespace resolved {

class data_type;

typedef const char *data_block;
typedef std::shared_ptr<const data_type> const_data_type_ptr;
typedef std::shared_ptr<value> value_ptr;

class data_type: public std::enable_shared_from_this<data_type> {
public:
    data_type();
    virtual int get_length() const = 0;
    virtual const_data_type_ptr get_base_type() const;
    virtual int get_array_depth() const;
    virtual bool is_proxy_type() const;
    virtual void print_repr(std::ostream &stream) const = 0;
    virtual void format(std::ostream &stream, data_block data) const = 0;
};

class abstract_data_type: public data_type {
private:
    std::string label;
public:
    abstract_data_type(std::string label);
    virtual int get_length() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, data_block data) const;
};

class int_data_type: public data_type {
public:
    virtual int get_length() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, data_block data) const;
};

class float_data_type: public data_type {
public:
    virtual int get_length() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, data_block data) const;
};

class bool_data_type: public data_type {
public:
    virtual int get_length() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, data_block data) const;
};

class array_data_type: public data_type {
private:
    const_data_type_ptr element_type;
    int length;
public:
    array_data_type(const_data_type_ptr element_type, int length);
    virtual int get_length() const;
    virtual const_data_type_ptr get_base_type() const;
    virtual int get_array_depth() const;
    virtual bool is_proxy_type() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, data_block data) const;

    virtual int get_array_length() const;
    const_data_type_ptr get_element_type() const;
    virtual value_ptr get_data_offset(value_ptr index) const;
};

class copy_array_data_proxy: public array_data_type {
public:
    copy_array_data_proxy(std::shared_ptr<const data_type> source_type, 
        int length);
    virtual bool is_proxy_type() const;
    virtual void print_repr(std::ostream &stream) const;
    virtual void format(std::ostream &stream, const char *data) const;

    virtual value_ptr get_data_offset(value_ptr index) const;
};

}
}