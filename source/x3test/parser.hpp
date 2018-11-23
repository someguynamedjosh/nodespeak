#pragma once

#include <boost/spirit/home/x3.hpp>
#include <string>

#include "ast.hpp"

namespace waveguide {
namespace parser {

using boost::spirit::x3::rule;

struct root_class;
using root_type = ast::Expression;
using root_rule_type = rule<root_class, root_type>;
BOOST_SPIRIT_DECLARE(root_rule_type)

root_type parse(std::string input);

}
}
