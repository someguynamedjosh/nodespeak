#include "ast_converter.hpp"

namespace waveguie {
namespace ast {

void AstConverter::operator()(int const&expr) const {
    data->current_value = int_literal(expr);
}

void AstConverter::operator()(double const&expr) const {
    data->current_value = double_literal(expr);
}

void AstConverter::operator()(bool const&expr) const {
    data->current_value = bool_literal(expr);
}

void AstConverter::operator()(SignedExpression const&expr) const {
    recurse(expr.value);
    if (expr.sign == '-') {
        SP<intr::Command> negate{new intr::Command(blt()->MUL)};
        negate.add_input(data->current_value);
        negate.add_input(int_literal(-1));
        SP<intr::Value> output{new intr::Value(blt()->UPCAST_WILDCARD)};
        declare_temp_var(output);
        data->current_value = output;
        negate.add_output(output);
        add_command(negate);
    }
}

void AstConverter::operator()(VariableExpression const&expr) const {
    // TODO: Add array access logic.
    data->current_value = lookup_var(expr.name);
}

void AstConverter::operator()(std::vector<Expression> const&expr) const {
    // TODO: Add array construction logic.
}

void AstConverter::operator()(SingleVarDec const&dec) const {
    // TODO: Add function output inline declaration logic.
}

void AstConverter::operator()(FunctionExpression const&expr) const {
    auto func = lookup_func(expr.function_name);
    SP<intr::Command> command{new intr::Command(func)};
    for (auto const&input : expr.inputs) {
        recurse(input);
        command.add_input(data->current_value);
    }
    for (auto const&output : expr.outputs) {
        recurse(output);
        command.add_output(data->current_value);
    }
    // TODO: Add lambda logic.
    add_command(command);
}

void AstConverter::operator()(OperatorListExpression const&expr) const {
    recurse(expr.start_value);
    std::string last_op{""};
    bool join{false};
    intr::Command *last_command{nullptr};
    for (auto const&operation : expr.operations) {
        if (operation.op_char != last_op || !join) {
            if (last_command) {
                SP<intr::Value> output{new intr::Value(blt()->UPCAST_WILDCARD)};
                declare_temp_var(output);
                last_command->add_output(output);
                add_command(last_command);
                data->current_value = output;
            }
            SP<intr::Scope> func{nullptr};
            auto const&c = operation.op_char;
            if (c == "+" || c == "-") {
                func = blt()->ADD;
                join = true;
            } else if (c == "*" || c == "/") {
                func = blt()->MUL;
                join = true;
            } else if (c == "%") {
                func = blt()->MOD;
                join = false;
            } else if (c == ">=") {
                func = blt()->GTE;
                join = false;
            } else if (c == "<=") {
                func = blt()->LTE;
                join = false;
            } else if (c == ">") {
                func = blt()->GT;
                join = false;
            } else if (c == "<") {
                func = blt()->LT;
                join = false;
            } else if (c == "==") {
                func = blt()->EQ;
                join = false;
            } else if (c == "!=") {
                func = blt()->NEQ;
                join = false;
            } else if (c == "band") {
                func = blt()->BAND;
                join = true;
            } else if (c == "bor") {
                func = blt()->BOR;
                join = true;
            } else if (c == "bxor") {
                func = blt()->BXOR;
                join = true;
            } else if (c == "and") {
                func = blt()->AND;
                join = false;
            } else if (c == "or") {
                func = blt()->OR;
                join = false;
            } else if (c == "xor") {
                func = blt()->XOR;
                join = false;
            }
            last_command = SP<intr::Command>{new intr::Command(func)};
            last_command->add_input(data->current_value);
        }
        recurse(operation.value);
        last_command->add_input(data->current_value);
    }
    if (last_command) {
        SP<intr::Value> output{new intr::Value(blt()->UPCAST_WILDCARD)};
        declare_temp_var(output);
        last_command->add_output(output);
        add_command(last_command);
        data->current_value = output;
    }
}

}
}