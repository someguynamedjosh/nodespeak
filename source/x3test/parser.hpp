#pragma once

#include <boost/spirit/home/x3.hpp>

#include "ast.hpp"

namespace waveguide {
namespace parser {

using boost::spirit::x3::rule;

struct root_class;
using root_type = rule<root_class, ast::Expression>;
BOOST_SPIRIT_DECLARE(root_type);

}
}
