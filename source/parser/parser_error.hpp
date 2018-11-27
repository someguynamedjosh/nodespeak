#pragma once

#include "parser.hpp"

namespace waveguide {
namespace parser {

namespace x3 = boost::spirit::x3;

struct error_handler {
    template<typename Iterator, typename Exception, typename Context>
    x3::error_handler_result on_error(Iterator &first, Iterator const& last,
        Exception const&error, Context const&context) {
        auto &error_handler = x3::get<x3::error_handler_tag>(context).get();
        std::string message = "Error! Expecting: " + error.which() + " here:";
        error_handler(error.where(), message);
        return x3::error_handler_result::fail;
    }
};

struct root_class: error_handler { };

}
}