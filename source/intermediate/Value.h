#ifndef _WAVEGUIDE_INTERMEDIATE_VALUE_H_
#define _WAVEGUIDE_INTERMEDIATE_VALUE_H_

#include <string>

namespace waveguide {
namespace intermediate {

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
    std::string repr();

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
}

#endif /* _WAVEGUIDE_INTERMEDIATE_VALUE_H_ */