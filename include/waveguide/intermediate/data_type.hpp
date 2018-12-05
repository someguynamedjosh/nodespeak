#pragma once

#include <memory>
#include <string>

namespace waveguide {
namespace intermediate {

class scope;
class value;

class data_type {
public:
    data_type();
    virtual int get_length() = 0;
    virtual std::shared_ptr<data_type> get_base_type();
    virtual bool is_proxy_type();
    virtual std::string repr() = 0;
    virtual std::string format(void *data) = 0;
};

class abstract_data_type: public data_type {
private:
    std::string label;
public:
    abstract_data_type(std::string label);
    virtual int get_length();
    virtual std::string repr();
    virtual std::string format(void *data);
};

class int_data_type: public data_type {
public:
    virtual int get_length();
    virtual std::string repr();
    virtual std::string format(void *data);
};

class float_data_type: public data_type {
public:
    virtual int get_length();
    virtual std::string repr();
    virtual std::string format(void *data);
};

class bool_data_type: public data_type {
public:
    virtual int get_length();
    virtual std::string repr();
    virtual std::string format(void *data);
};

class array_data_type: public data_type {
private:
    std::shared_ptr<data_type> elementType;
    int length;
public:
    array_data_type(std::shared_ptr<data_type> elementType, int length);
    virtual int get_length();
    virtual std::shared_ptr<data_type> get_base_type();
    virtual std::string repr();
    virtual std::string format(void *data);

    virtual int getArrayLength();
    std::shared_ptr<data_type> get_element_type();
    virtual std::shared_ptr<value> get_data_offset(std::shared_ptr<value> index);
};

class copy_array_data_proxy: public array_data_type {
public:
    copy_array_data_proxy(std::shared_ptr<data_type> sourceType, int length);
    virtual bool is_proxy();
    virtual std::string repr();
    virtual std::string format(void *data);

    virtual std::shared_ptr<value> get_data_offset(std::shared_ptr<value> index);
};

}
}
