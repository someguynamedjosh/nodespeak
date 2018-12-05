#pragma once

#include <memory>
#include <string>

namespace waveguide {
namespace intermediate {

class data_type;

class value {
private:
    std::shared_ptr<data_type> type;
    void *data;
    bool value_known{false};
public:
    value(std::shared_ptr<data_type> type);
    value(std::shared_ptr<data_type> type, void *data);
    ~value();
    std::string repr();

    std::shared_ptr<data_type> get_type();
    void set_type(std::shared_ptr<data_type> new_type);
    bool is_proxy();
    value &get_real_value();

    bool is_value_known();
    void set_value_known(bool is_known);
    value create_known_copy();
    void *get_data();

    float *data_as_float();
    int *data_as_int();
    bool *data_as_bool();
};

}
}
