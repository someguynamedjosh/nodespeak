#include "passes.hpp"

#include <waveguide/intermediate/builtins.hpp>
#include <waveguide/intermediate/metastructure.hpp>

#include "intermediate/util.hpp"

namespace waveguide {
namespace squash {

// Alright, so here's the casting rules:
//
// Float > Int > Bool. Whenever you have two input variables that need to have
// the same type, pick the biggest type of the two. (E.G. Int + Float means
// cast the first argument to Float, then do the operation.)
//
// If you have TYPE1 and [A]TYPE2, the first parameter should be treated as an
// array of size 1. For example, `[1, 2] + 10` will be treated the same as
// `[1, 2] + [10]`.
//
// If you have [1]TYPE1 and [A]TYPE2, the first array is duplicated so that
// it has length A. For example, `[1, 2] + [10]` should be cast to 
// `[1, 2] + [10, 10]`. Internally, this should use a proxy data type so that
// the array does not have to be physically duplicated. Instead, the same memory
// location is accessed regardless of the index that is being accessed.
// 
// If you have [A]TYPE1 and [B]TYPE2, and neither A nor B are 1, the cast is 
// ambiguous. When there is only one element in one of the arrays (as in the
// previous rule), it is easy to just stretch it to the size of the other array.
// However, in this case, it is ambiguous what method should be used to stretch
// one array to fit the other. (Copy it? Stretch the elements?) Any method
// picked would only be helpful in a marginal set of circumstances. Thus, it is
// up to the programmer to either match the array sizes themselves or reducing
// one of the arrays to a size one.
//
// If you have [A]TYPE1 and [A]TYPE2, the rules for matching TYPE1 and TYPE2
// are applied to all elements of TYPE1 and TYPE2. For example, 
// `[1, 2] + [0.3, 0.4]` should be cast to `[1.0, 2.0] + [0.3, 0.4]`.
// 
// If you have TYPE1 and TYPE2, and both types are elementary data types, use
// the hierarchy Float > Int > Bool. Pick the biggest type and cast the smaller
// parameter to that type. For example, `5 + TRUE` becomes `5 + 1`.
//
// SOME MORE COMPLETE / COMPLICATED EXAMPLES:
//
//    [5]Float + [3]Float: [A]TYPE1 + [B]TYPE2
// -> ambiguous
//
//    [4]Float + [4]Int: [A]TYPE1 + [A]TYPE2.
// -> Float + Int           : TYPE1 + TYPE2.
// -> Float + Float         : Float is biggest type
// -> [4]Float + [4]Float   : Apply cast for each element.
//
//    Float + [4]Int        : TYPE1 + [A]TYPE2.
// -> [1]Float + [4]Int     : [1]TYPE1 + [A]TYPE2.
// -> [4]Float + [4]Int     : [A]TYPE1 + [A]TYPE2.
// -> Follow previous example.
//
//    [4]Int + [4][2]Int    : [A]TYPE1 + [A]TYPE2 (TYPE1=Int, TYPE2=[2]Int)
// -> Int + [2]Int          : TYPE1 + [A]TYPE2
// -> [1]Int + [2]Int       : [1]TYPE1 + [A]TYPE2
// -> [2]Int + [2]Int       : [A]TYPE1 + [A]TYPE2
// -> [4][2]Int + [4][2]Int : Apply cast for each element.
//
//    [1][2]Int + [4][2]Int : [1]TYPE1 + [A]TYPE2 (TYPE1=[2]Int, TYPE2=[2]Int)
// -> [4][2]Int + [4][2]Int : Copy the array.

intr::value_ptr cast_value(intr::scope_ptr context, intr::value_ptr input, 
    intr::const_data_type_ptr target) {
    intr::const_data_type_ptr input_type = input->get_type();
    std::vector<int> in_sizes, target_sizes;
    while (auto array_type 
        = std::dynamic_pointer_cast<const intr::array_data_type>(input_type)) {
        in_sizes.push_back(array_type->get_array_length());
        input_type = array_type->get_element_type();
    }
    while (auto array_type 
        = std::dynamic_pointer_cast<const intr::array_data_type>(target)) {
        target_sizes.push_back(array_type->get_array_length());
        target = array_type->get_element_type();
    }
    int in_depth = in_sizes.size(), target_depth = target_sizes.size();
    // There is no way to cast an array to a single value.
    if (in_depth > target_depth) { 
        // TODO: Error message
        return nullptr;
    } 

    // Proxy type is a proxy used to access the data before doing actual data
    // type casting. This is used to resolve array size differences. The output
    // type is the type the output will have. It will be compatible with the
    // target type, but may use proxies instead of actual array types for
    // efficiency.
    intr::const_data_type_ptr output_proxy_type = target->get_base_type(),
        output_type = target->get_base_type();
    bool proxy_needed = false;
    for (unsigned int j = target_depth; j > 0; j--) {
        unsigned int i = j - 1;
        // If the input has a bare type or an array of size 1 that needs to be
        // matched to a larger array, do a copy proxy to extend its size.
        if (i >= in_sizes.size() || (in_sizes[i] == 1 && target_sizes[i] > 1)) {
            output_type = std::make_shared<intr::array_data_type>(
                output_type, 1
            );
            output_proxy_type = std::make_shared<intr::copy_array_data_proxy>(
                output_proxy_type, target_sizes[i]
            );
            proxy_needed = true;
        } else if (in_sizes[i] == target_sizes[i]) {
            output_type = std::make_shared<intr::array_data_type>(
                output_type, target_sizes[i]
            );
            output_proxy_type = std::make_shared<intr::array_data_type>(
                output_proxy_type, target_sizes[i]
            );
        } else {
            // TODO: Error message. The two array sizes cannot be resolved
            // with each other.
            return nullptr;
        }
    }

    auto in_base = input_type->get_base_type(), 
        target_base = target->get_base_type();
    intr::scope_ptr convert_func{nullptr};
    if (in_base == intr::blt()->FLOAT) {
        if (target_base == intr::blt()->INT) {
            convert_func = intr::blt()->FTOI;
        } else if (target_base == intr::blt()->BOOL) {
            convert_func = intr::blt()->FTOB;
        }
    } else if (in_base == intr::blt()->INT) {
        if (target_base == intr::blt()->FLOAT) {
            convert_func = intr::blt()->ITOF;
        } else if (target_base == intr::blt()->BOOL) {
            convert_func = intr::blt()->ITOB;
        }
    } else if (in_base == intr::blt()->BOOL) {
        if (target_base == intr::blt()->INT) {
            convert_func = intr::blt()->BTOI;
        } else if (target_base == intr::blt()->FLOAT) {
            convert_func = intr::blt()->BTOF;
        }
    }
    intr::value_ptr output{nullptr};
    if (convert_func) {
        output = std::make_shared<intr::value>(output_type);
        context->declare_temp_var(output);
        auto convert{std::make_shared<intr::command>(convert_func)};
        convert->add_input(input);
        convert->add_output(output);
        context->add_command(convert);
    } else {
        output = input;
    }

    if (proxy_needed) {
        auto proxied{std::make_shared<intr::value>(output_proxy_type, output)};
        context->declare_temp_var(proxied);
        return proxied;
    } else {
        return output;
    }
}

void cast_command(intr::scope_ptr context, intr::command_ptr command) {
    const std::vector<intr::value_ptr> &ins = command->get_inputs(), 
        &outs = command->get_outputs();
    const std::vector<intr::command_lambda> &lambdas = command->get_lambdas();

    std::map<std::string, std::vector<intr::data_type_ptr>> type_map;
    std::map<std::string, std::vector<double>> value_map;

    auto algebra = [&](auto )

    auto unravel = [&](auto real_value, auto param_value) {
        auto real_type = std::dynamic_pointer_cast<intr::unresolved_vague_type>(
            real_value->get_type();
        )->get_vague_type();
        auto param_type = std::dynamic_pointer_cast
            <intr::unresolved_vague_type>(
                param_value->get_type()
        )->get_vague_type();
        while (auto array_type = std::dynamic_pointer_cast
            <intr::vague_array_data_type>(param_type)) {
            param_type = array_type->get_element_type();
            if (auto real_array_type = std::dynamic_pointer_cast
                <intr::vague_array_data_type>(real_type)) {
                real_type = real_array_type->get_element_type();
            }
        }
    }
}

void cast_pass(intr::scope_ptr scope) {
    // TODO: Do something.
}

}
}