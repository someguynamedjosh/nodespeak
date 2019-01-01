#include "ast_converter.hpp"

#include <waveguide/convert/convert.hpp>
#include <waveguide/intermediate/metastructure.hpp>

namespace waveguide {
namespace convert {

ast_conversion_exception::ast_conversion_exception(std::string message)
    : message(message) { }

const char *ast_conversion_exception::what() const throw() {
    return message.c_str();
}

conversion_result convert_ast(ast::root_type const&root) {
    conversion_result result{};
    ast::ast_converter converter{};
    try {
        converter.start(root);
        result.converted_scope = converter.get_result();
        result.success = true;
        result.error_message = "";
    } catch (ast_conversion_exception &e) {
        result.converted_scope = nullptr;
        result.success = false;
        result.error_message = std::string{e.what()};
    }
    return result;
}

}
}