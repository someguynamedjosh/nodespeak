#pragma once

#include <cassert>
#include <memory>
#include <string>

namespace waveguide {
namespace intermediate {

class data_type;

class value {
private:
    std::shared_ptr<const data_type> type;
    std::shared_ptr<char[]> data;
    bool value_known{false};
public:
    value(std::shared_ptr<const data_type> type);
    value(std::shared_ptr<const data_type> type, std::shared_ptr<char[]> data);
    value(std::shared_ptr<const data_type> type, std::shared_ptr<value> target);
    template<typename T>
    value(std::shared_ptr<const data_type> type, std::shared_ptr<T> in_data)
        : type{type}, value_known{true} {
        assert(!type->is_proxy_type());
        data = std::reinterpret_pointer_cast<char[]>(in_data);
    }

    std::shared_ptr<const data_type> get_type() const;
    void set_type(std::shared_ptr<const data_type> new_type);
    bool is_proxy() const;
    value const&get_real_value() const;

    bool is_value_known() const;
    void set_value_known(bool is_known);
    value create_known_copy() const;
    const std::shared_ptr<char[]> get_data() const;
    std::shared_ptr<char[]> get_data();

    const std::shared_ptr<float> data_as_float() const;
    const std::shared_ptr<int> data_as_int() const;
    const std::shared_ptr<bool> data_as_bool() const;
    std::shared_ptr<float> data_as_float();
    std::shared_ptr<int> data_as_int();
    std::shared_ptr<bool> data_as_bool();
};

}
}
