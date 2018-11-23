#pragma once

#include <boost/spirit/home/x3.hpp>
#include <string>

#include "ast.hpp"

namespace waveguide {
namespace parser {

using boost::spirit::x3::rule;

struct root_class;
using root_type = ast::Statement;
using root_rule_type = rule<root_class, root_type>;
BOOST_SPIRIT_DECLARE(root_rule_type)

struct ParseResult {
    root_type ast;
    int error = 0;
};

ParseResult parse(std::string input);

}
}
