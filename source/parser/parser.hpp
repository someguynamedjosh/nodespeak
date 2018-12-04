#pragma once

#include <boost/spirit/home/x3.hpp>
#include <string>
#include <vector>

#include <waveguide/parser/parser.hpp>

namespace waveguide {
namespace parser {

using boost::spirit::x3::rule;

struct root_class;
using root_rule_type = rule<root_class, ast::root_type>;
BOOST_SPIRIT_DECLARE(root_rule_type)

struct error_handler_tag;

parse_result parse(std::string input);

}
}
