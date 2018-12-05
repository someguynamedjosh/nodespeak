#pragma once

#include <boost/core/enable_if.hpp>
#include <waveguide/intermediate/builtins.hpp>
#include <waveguide/intermediate/metastructure.hpp>
#include <waveguide/parser/ast.hpp>
#include <memory>

namespace waveguide {
namespace ast {

namespace intr = waveguide::intermediate;
template<typename T>
using SP = std::shared_ptr<T>;

inline SP<intr::builtins> blt() {
    return intr::builtins::get_instance();
}

inline SP<intr::value> int_literal(const int value) {
    return SP<intr::value>{new intr::value(blt()->INT, new int{value})};
}

inline SP<intr::value> double_literal(const double value) {
    return SP<intr::value>{new intr::value(blt()->FLOAT, new double{value})};
}

inline SP<intr::value> bool_literal(const bool value) {
    return SP<intr::value>{new intr::value(blt()->BOOL, new bool{value})};
}

struct AccessResult {
    SP<intr::value> root_val, offset;
    SP<intr::data_type> final_type;
};

struct AstConverter: boost::static_visitor<> {
    struct ConverterData {
        SP<intr::scope> current_scope;
        SP<intr::value> current_value;
        SP<intr::data_type> current_type;
    };
    SP<ConverterData> data;
    mutable std::vector<SP<ConverterData>> stack;

    AstConverter();
    AstConverter(SP<ConverterData> data): data{data} { }

    // Utility methods.
    void push_stack() const;
    void pop_stack() const;
    AccessResult find_access_result(ast::variable_expression const&expr) const;
    void copy_value_to_expr(SP<intr::value> from, 
        ast::variable_expression const& to) const;
    void copy_value_from_expr(ast::variable_expression const& from,
        SP<intr::value> to) const;
    SP<intr::value> lookup_var(std::string name) const;
    SP<intr::scope> lookup_func(std::string name) const;
    SP<intr::data_type> lookup_type(std::string name) const;
    void add_command(SP<intr::command> command) const;
    void declare_temp_var(SP<intr::value> var) const;
    
    // Parses statements into the current scope.
    void operator()(std::vector<statement> const&stats) const;
    void operator()(function_statement const&stat) const;
    void operator()(assign_statement const&stat) const;
    void operator()(var_dec_statement const&stat) const;
    void operator()(Plainvar_dec const&dec) const;
    void operator()(init_var_dec const&dec) const;
    void operator()(return_statement const&stat) const;

    void operator()(int const&expr) const;
    void operator()(double const&expr) const;
    void operator()(bool const&expr) const;
    void operator()(signed_expression const&expr) const;
    void operator()(variable_expression const&expr) const;
    void operator()(std::vector<expression> const&expr) const;
    void operator()(single_var_dec const&dec) const;
    void operator()(function_expression const&expr) const;
    void operator()(operator_list_expression const&expr) const;

    void operator()(function_parameter_dec const&dec) const;
    void operator()(function_dec const&dec) const;
    void operator()(data_type const&type) const;

    template<typename T>
    struct has_visit_method {
    private:
        typedef std::true_type yes;
        typedef std::false_type no;
        
        template<typename U> static auto test(int) -> decltype(
            std::declval<U>().apply_visitor(std::declval<AstConverter>()), 
            yes());
        template<typename> static no test(...);

    public:
        static constexpr bool value
            = std::is_same<decltype(test<T>(0)),yes>::value;
    };

    #pragma GCC diagnostic ignored "-Wunused-parameter"
    template<typename Visitable>
    typename boost::enable_if<has_visit_method<Visitable>, void>::type
    recurse(Visitable &to_convert) const {
        boost::apply_visitor(AstConverter{data}, to_convert);
    }

    template<typename Visitable>
    typename boost::disable_if<has_visit_method<Visitable>, void>::type
    recurse(Visitable &to_convert) const {
        (*this)(to_convert);
    }
    #pragma GCC diagnostic pop // Restore command-line options.
};

}
}