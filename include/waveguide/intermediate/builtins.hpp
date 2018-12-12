#pragma once

#include <memory>

namespace waveguide {
namespace intermediate {

class data_type;
class scope;

class builtins {
private:
    builtins();
    static std::shared_ptr<builtins> instance;
public:
    static std::shared_ptr<builtins> get_instance();
    void add_to_scope(std::shared_ptr<scope> scope);
    std::shared_ptr<data_type> INT, FLOAT, BOOL, UPCAST_WILDCARD, ANY_WILDCARD;
    std::shared_ptr<scope> ADD, MUL, RECIP, MOD, BAND, BOR, BXOR,
        ITOF, BTOF, BTOI, ITOB, FTOI, FTOB,
        COPY, COPY_TO_INDEX, COPY_FROM_INDEX, LOG, RETURN,
        EQ, NEQ, LTE, GTE, LT, GT, AND, OR, XOR;
};

}
}
