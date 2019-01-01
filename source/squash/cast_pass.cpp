#include "passes.hpp"

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

void cast_pass(SP<intr::scope> scope) {
    // TODO: Do something.
}

}
}