#include <boost/spirit/home/x3.hpp>
#include <iostream>

#include "rules.hpp"
#include "config.hpp"

namespace waveguide {
namespace parser {

namespace x3 = boost::spirit::x3;

BOOST_SPIRIT_INSTANTIATE(root_rule_type, iterator_type, context_type)

root_type parse(std::string input) {
    ast::Expression result;
    
    bool success = phrase_parse(input.begin(), input.end(), root_rule,
        x3::ascii::space, result);
    std::cout << success << std::endl;
    return result;
}

}
}