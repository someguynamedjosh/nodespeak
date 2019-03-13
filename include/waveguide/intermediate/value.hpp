#pragma once

#include <cassert>
#include <memory>
#include <string>
#include <vector>

namespace waveguide {
namespace intermediate {

class data_type;
class value;
class value_accessor;

typedef std::shared_ptr<const data_type> const_data_type_ptr;
typedef char *data_block_ptr;
typedef const char *const_data_block_ptr;
typedef std::shared_ptr<char[]> shared_data_block_ptr;
typedef std::shared_ptr<value> value_ptr;
typedef std::shared_ptr<const value> const_value_ptr;
typedef std::shared_ptr<value_accessor> value_accessor_ptr;
typedef std::shared_ptr<const value_accessor> const_value_accessor_ptr;

class value {
private:
    const_data_type_ptr type;
    shared_data_block_ptr data;
    bool value_known{false};
public:
    value(const_data_type_ptr type);
    value(const_data_type_ptr type, shared_data_block_ptr data);
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

    const_data_block_ptr get_data() const;
    float const&data_as_float() const;
    int const&data_as_int() const;
    bool const&data_as_bool() const;
    data_block_ptr get_data();
    float &data_as_float();
    int &data_as_int();
    bool &data_as_bool();
};

class value_accessor {
private:
    value_ptr root_value{nullptr};
    std::vector<const_value_ptr> subparts{};

public:
    value_accessor();
    value_accessor(value_ptr root_value);

    void set_root_value(value_ptr root_value);
    value_ptr get_root_value() const;

    void add_subpart(const_value_ptr subpart);
    std::vector<const_value_ptr> const&get_subparts() const;

    bool is_value_known() const;
    const_data_type_ptr get_type() const;

    const_data_block_ptr get_data() const;
    float const&data_as_float() const;
    int const&data_as_int() const;
    bool const&data_as_bool() const;
    data_block_ptr get_data();
    float &data_as_float();
    int &data_as_int();
    bool &data_as_bool();
};

}
}
