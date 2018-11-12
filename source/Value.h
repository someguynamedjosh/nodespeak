#ifndef _VALUE_H_
#define _VALUE_H_

#include <string>

namespace Com {

class DataType;

class Value {
private:
    DataType &type;
    void *data;
    bool valueKnown{false};
public:
    Value(DataType &type);
    Value(DataType &type, void *data);
    ~Value();
    std::string format();

    DataType &getType();
    void setType(DataType &newType);
    bool isProxy();
    Value &getRealValue();

    bool isValueKnown();
    void setValueKnown(bool isKnown);
    Value createKnownCopy();
    void *getData();

    float *dataAsFloat();
    int *dataAsInt();
    bool *dataAsBool();
};

}

#endif /* _VALUE_H_ */