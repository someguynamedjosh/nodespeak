#ifndef _DATA_TYPE_H_
#define _DATA_TYPE_H_

namespace Com {

class DataType {
public:
    DataType();
    virtual int getLength() = 0;
    virtual DataType *getBaseType();
    virtual bool isProxyType();
};

class IntDataType: public DataType {
public:
    virtual int getLength();
};

class FloatDataType: public DataType {
public:
    virtual int getLength();
};

class BoolDataType: public DataType {
public:
    virtual int getLength();
};

class ArrayDataType: public DataType {
private:
    DataType *elementType;
    int length;
public:
    ArrayDataType(DataType *elementType, int length);
    virtual int getLength();
    virtual DataType *getBaseType();

    virtual int getArrayLength();
    DataType *getElementType();
};

class CopyArrayDataProxy: public ArrayDataType {
public:
    CopyArrayDataProxy(DataType *sourceType, int length);
    virtual bool isProxy();
};

}

#endif /* _DATA_TYPE_H_ */