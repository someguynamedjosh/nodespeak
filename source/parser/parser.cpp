#include <boost/spirit/home/x3.hpp>
#include <iostream>

#include "parser_rules.hpp"

namespace waveguide {
namespace parser {

namespace x3 = boost::spirit::x3;

using iterator_type = std::string::const_iterator;
using error_handler_type = x3::error_handler<iterator_type>;
using position_cache = x3::position_cache<std::vector<iterator_type>>;
using context_type = x3::context<
    x3::error_handler_tag, 
    std::reference_wrapper<error_handler_type> const, 
    x3::phrase_parse_context<x3::ascii::space_type>::type>;

BOOST_SPIRIT_INSTANTIATE(
    root_rule_type, 
    iterator_type,
    context_type)

parse_result parse(std::string input) {
    parse_result result;
    iterator_type start = input.begin(), end = input.end();
    error_handler_type error_handler(start, end, std::cerr);

    auto parser = x3::with<x3::error_handler_tag>(std::ref(error_handler))[
        root_rule
    ];
    
    bool success = phrase_parse(start, end, parser, skipper, result.ast);

    if (start != end) {
        std::cerr << "Parser exited prematurely, the following code was not "
            << "parsed:" << std::endl;
        for (auto i = start; i != end; i++) {
            std::cerr << *i;
        }
        std::cerr << std::endl;
    }
    success &= start == end;
    result.error = success ? 0 : 1;
    return result;
}

}
}