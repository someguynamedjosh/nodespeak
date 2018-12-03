#pragma once

#include <memory>
#include <string>

namespace waveguide {
namespace intermediate {

class Scope;
class Value;

class DataType {
public:
    DataType();
    virtual int get_length() = 0;
    virtual std::shared_ptr<DataType> get_base_type();
    virtual bool is_proxy_type();
    virtual std::string repr() = 0;
    virtual std::string format(void *data) = 0;
};

class AbstractDataType: public DataType {
private:
    std::string label;
public:
    AbstractDataType(std::string label);
    virtual int get_length();
    virtual std::string repr();
    virtual std::string format(void *data);
};

class IntDataType: public DataType {
public:
    virtual int get_length();
    virtual std::string repr();
    virtual std::string format(void *data);
};

class FloatDataType: public DataType {
public:
    virtual int get_length();
    virtual std::string repr();
    virtual std::string format(void *data);
};

class BoolDataType: public DataType {
public:
    virtual int get_length();
    virtual std::string repr();
    virtual std::string format(void *data);
};

class ArrayDataType: public DataType {
private:
    std::shared_ptr<DataType> elementType;
    int length;
public:
    ArrayDataType(std::shared_ptr<DataType> elementType, int length);
    virtual int get_length();
    virtual std::shared_ptr<DataType> get_base_type();
    virtual std::string repr();
    virtual std::string format(void *data);

    virtual int getArrayLength();
    std::shared_ptr<DataType> get_element_type();
    virtual std::shared_ptr<Value> get_data_offset(std::shared_ptr<Value> index);
};

class CopyArrayDataProxy: public ArrayDataType {
public:
    CopyArrayDataProxy(std::shared_ptr<DataType> sourceType, int length);
    virtual bool is_proxy();
    virtual std::string repr();
    virtual std::string format(void *data);

    virtual std::shared_ptr<Value> get_data_offset(std::shared_ptr<Value> index);
};

}
}