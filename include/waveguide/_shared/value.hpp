#pragma once

#include <memory>
#include <string>
#include <vector>

namespace waveguide {
namespace _shared {

typedef char *data_block_ptr;
typedef const char *const_data_block_ptr;
typedef std::shared_ptr<char[]> shared_data_block_ptr;

template<typename Metatype>
class value {
private:
    using const_metatype_ptr = std::shared_ptr<const Metatype>;
    using value_ptr = std::shared_ptr<value<Metatype>>;
    // TODO: Remove this value in production builds.
    std::string debug_label;
    const_metatype_ptr type;
    shared_data_block_ptr data;
    bool value_known{false};
public:
    value(const_metatype_ptr type);
    value(const_metatype_ptr type, shared_data_block_ptr data);
    value(const_metatype_ptr type, data_block_ptr data);
    value(const_metatype_ptr type, value_ptr target);
    template<typename T>
    value(const_metatype_ptr type, std::shared_ptr<T> in_data)
        : type{type}, value_known{true} {
        assert(!type->is_proxy_type());
        data = std::reinterpret_pointer_cast<char[]>(in_data);
    }
    void set_debug_label(std::string label);
    std::string get_debug_label() const;

    const_metatype_ptr get_type() const;
    void set_type(const_metatype_ptr new_type);
    bool is_proxy() const;
    value<Metatype> const&get_real_value() const;

    bool is_value_known() const;
    void set_value_known(bool is_known);
    value<Metatype> create_known_copy() const;

    const_data_block_ptr get_data() const;
    float const&data_as_float() const;
    int const&data_as_int() const;
    bool const&data_as_bool() const;
    data_block_ptr get_data();
    float &data_as_float();
    int &data_as_int();
    bool &data_as_bool();
};

template<typename ValueType>
class value_accessor {
private:
    // TODO: Remove this value in production builds.
    using value_ptr = std::shared_ptr<ValueType>;
    using this_type = value_accessor<ValueType>;
    using const_metatype_ptr = typename ValueType::const_metatype_ptr;
    using const_value_accessor_ptr = std::shared_ptr<const this_type>;
    value_ptr root_value{nullptr};
    std::vector<const_value_accessor_ptr> subparts{};
public:
    value_accessor();
    value_accessor(value_ptr root_value);
    std::string get_debug_label() const;

    void set_root_value(value_ptr root_value);
    value_ptr get_root_value() const;

    void add_subpart(const_value_accessor_ptr subpart);
    std::vector<const_value_accessor_ptr> const&get_subparts() const;

    bool is_value_known() const;
    const_metatype_ptr get_type() const;

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