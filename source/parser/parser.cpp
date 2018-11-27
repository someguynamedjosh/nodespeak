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

ParseResult parse(std::string input) {
    ParseResult result;
    iterator_type start = input.begin(), end = input.end();
    position_cache positions{start, end};

    auto parser = x3::with<position_cache_tag>(positions)[root_rule];
    
    bool success = phrase_parse(start, end, parser, x3::ascii::space, 
        result.ast);

    success &= start == end;
    result.error = success ? 0 : 1;
    return result;
}

}
}