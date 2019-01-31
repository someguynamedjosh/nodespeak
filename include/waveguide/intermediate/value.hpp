#pragma once

#include <cassert>
#include <memory>
#include <string>

namespace waveguide {
namespace intermediate {

class data_type;
class value;

typedef std::shared_ptr<const data_type> const_data_type_ptr;
typedef std::shared_ptr<char[]> data_block_ptr;
typedef std::shared_ptr<value> value_ptr;
typedef std::shared_ptr<value_accessor> value_accessor_ptr;

class value {
private:
    const_data_type_ptr type;
    data_block_ptr data;
    bool value_known{false};
public:
    value(const_data_type_ptr type);
    value(const_data_type_ptr type, data_block_ptr data);
    value(const_data_type_ptr type, value_ptr target);
    template<typename T>
    value(const_data_type_ptr type, std::shared_ptr<T> in_data)
        : type{type}, value_known{true} {
        assert(!type->is_proxy_type());
        data = std::reinterpret_pointer_cast<char[]>(in_data);
    }

    const_data_type_ptr get_type() const;
    void set_type(const_data_type_ptr new_type);
    bool is_proxy() const;
    value const&get_real_value() const;

    bool is_value_known() const;
    void set_value_known(bool is_known);
    value create_known_copy() const;
    const data_block_ptr get_data() const;
    data_block_ptr get_data();

    const std::shared_ptr<float> data_as_float() const;
    const std::shared_ptr<int> data_as_int() const;
    const std::shared_ptr<bool> data_as_bool() const;
    std::shared_ptr<float> data_as_float();
    std::shared_ptr<int> data_as_int();
    std::shared_ptr<bool> data_as_bool();
};

class value_accessor {
private:
    value_ptr root_value{nullptr};
    std::vector<value_ptr> subparts{};
public:
    value_accessor();
    value_accessor(value_ptr root_value);

    void set_root_value(value_ptr root_value);
    value_ptr get_root_value() const;

    void add_subpart(value_ptr subpart);
    std::vector<value_ptr> const&get_subparts() const;
}

}
}
