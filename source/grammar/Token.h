#ifndef _WAVEGUIDE_GRAMMAR_TOKEN_H_
#define _WAVEGUIDE_GRAMMAR_TOKEN_H_

#include <string>

namespace waveguide {
namespace grammar {

class Token {
public:
    Token();
    virtual std::string repr() = 0;
};

}
}

#endif /* _WAVEGUIDE_GRAMMAR_TOKEN_H_ */