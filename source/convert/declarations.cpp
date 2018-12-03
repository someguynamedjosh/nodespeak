#include "ast_converter.hpp"

namespace waveguide {
namespace ast {

void AstConverter::operator()(FunctionParameterDec const&dec) const {
    // TODO: add logic.
}

void AstConverter::operator()(FunctionDec const&dec) const {
    // TODO: add logic.
}

void AstConverter::operator()(DataType const&type) const {
    // TODO: add logic for array types.
    data->current_type = lookup_type(type.name);
}

}
}