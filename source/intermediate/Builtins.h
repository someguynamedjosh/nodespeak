#ifndef _INTERMEDIATE_BUILTINS_H_
#define _INTERMEDIATE_BUILTINS_H_

#include <memory>

namespace waveguide {
namespace intermediate {

class DataType;
class Scope;

class Builtins {
private:
    Builtins();
    static std::shared_ptr<Builtins> instance;
public:
    static std::shared_ptr<Builtins> getInstance();
    void addToScope(std::shared_ptr<Scope> scope);
    std::shared_ptr<DataType> INT, FLOAT, BOOL, UPCAST_WILDCARD, ANY_WILDCARD;
    std::shared_ptr<Scope> ADD, MUL, RECIP, MOD, BAND, BOR, BXOR,
        ITOF, BTOF, BTOI, ITOB, FTOI, FTOB,
        COPY, LOG,
        EQ, NEQ, LTE, GTE, LT, GT, AND, OR, XOR;
};

}
}

#endif /* _INTERMEDIATE_BUILTINS_H_ */