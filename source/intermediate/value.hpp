#pragma once

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

    std::shared_ptr<DataType> get_type();
    void set_type(std::shared_ptr<DataType> newType);
    bool is_proxy();
    Value &get_real_value();

    bool is_value_known();
    void set_value_known(bool isKnown);
    Value create_known_copy();
    void *get_data();

    float *data_as_float();
    int *data_as_int();
    bool *data_as_bool();
};

}
}
