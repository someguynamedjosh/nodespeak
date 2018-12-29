#pragma once

#include <memory>
#include <string>

namespace waveguide {
namespace intermediate {

class data_type;

class value {
private:
    std::shared_ptr<const data_type> type;
    void *data;
    bool value_known{false};
public:
    value(std::shared_ptr<const data_type> type);
    value(std::shared_ptr<const data_type> type, void *data);
    ~value();

    std::shared_ptr<const data_type> get_type() const;
    void set_type(std::shared_ptr<const data_type> new_type);
    bool is_proxy() const;
    value const&get_real_value() const;

    bool is_value_known() const;
    void set_value_known(bool is_known);
    value create_known_copy() const;
    const void *get_data() const;
    void *get_data();

    const float *data_as_float() const;
    const int *data_as_int() const;
    const bool *data_as_bool() const;
    float *data_as_float();
    int *data_as_int();
    bool *data_as_bool();
};

}
}
