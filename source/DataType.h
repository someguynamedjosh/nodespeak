#ifndef _DATA_TYPE_H_
#define _DATA_TYPE_H_

#include <string>

namespace Com {

class Scope;
class Value;

class DataType {
public:
    DataType();
    virtual int getLength() = 0;
    virtual DataType &getBaseType();
    virtual bool isProxyType();
    virtual std::string format(void *data);
};

class IntDataType: public DataType {
public:
    virtual int getLength();
    virtual std::string format(void *data);
};

class FloatDataType: public DataType {
public:
    virtual int getLength();
    virtual std::string format(void *data);
};

class BoolDataType: public DataType {
public:
    virtual int getLength();
    virtual std::string format(void *data);
};

class ArrayDataType: public DataType {
private:
    DataType &elementType;
    int length;
public:
    ArrayDataType(DataType &elementType, int length);
    virtual int getLength();
    virtual DataType &getBaseType();
    virtual std::string format(void *data);

    virtual int getArrayLength();
    DataType &getElementType();
    virtual Value &getDataOffset(Value &index);
};

class CopyArrayDataProxy: public ArrayDataType {
public:
    CopyArrayDataProxy(DataType &sourceType, int length);
    virtual bool isProxy();
    virtual std::string format(void *data);

    virtual Value &getDataOffset(Value &index);
};

}

#endif /* _DATA_TYPE_H_ */