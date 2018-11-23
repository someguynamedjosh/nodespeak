#include <boost/spirit/home/x3.hpp>
#include <iostream>

#include "parser_rules.hpp"

namespace waveguide {
namespace parser {

namespace x3 = boost::spirit::x3;

using iterator_type = std::string::const_iterator;
using context_type = x3::phrase_parse_context<x3::ascii::space_type>::type;

BOOST_SPIRIT_INSTANTIATE(
    root_rule_type, 
    iterator_type,
    context_type)

root_type parse(std::string input) {
    root_type result;
    iterator_type start = input.begin(), end = input.end();
    
    bool success = phrase_parse(start, end, root_rule, x3::ascii::space, 
        result);

    success &= start == end;
    std::cout << success << std::endl;
    return result;
}

}
}