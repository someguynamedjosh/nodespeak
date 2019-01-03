#pragma once

#include <memory>

namespace waveguide {
namespace intermediate {

class builtins;
class data_type;
class scope;

typedef std::shared_ptr<builtins> builtins_ptr;
typedef std::shared_ptr<data_type> data_type_ptr;
typedef std::shared_ptr<scope> scope_ptr;

class builtins {
private:
    builtins();
    static builtins_ptr instance;
public:
    static builtins_ptr get_instance();
    void add_to_scope(scope_ptr scope);
    data_type_ptr INT, FLOAT, BOOL, DEDUCE_LATER;
    scope_ptr
        ADD, MUL, RECIP, MOD, BAND, BOR, BXOR,
        ITOF, BTOF, BTOI, ITOB, FTOI, FTOB,
        EQ, NEQ, LTE, GTE, LT, GT, AND, OR, XOR,
        COPY, COPY_TO_INDEX, COPY_FROM_INDEX, RETURN,
        LOG, DEF, IF, FOR, FOR_EACH, WHILE;
};

}
}
