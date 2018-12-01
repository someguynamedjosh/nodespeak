#ifndef _WAVEGUIDE_INTERMEDIATE_VALUE_H_
#define _WAVEGUIDE_INTERMEDIATE_VALUE_H_

#include <memory>
#include <string>

namespace waveguide {
namespace intermediate {

class DataType;

class Value {
private:
    std::shared_ptr<DataType> type;
    void *data;
    bool valueKnown{false};
public:
    Value(std::shared_ptr<DataType> type);
    Value(std::shared_ptr<DataType> type, void *data);
    ~Value();
    std::string repr();

    std::shared_ptr<DataType> getType();
    void setType(std::shared_ptr<DataType> newType);
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