#pragma once

#include <waveguide/vague/builtins.hpp>

waveguide::vague::builtins_ptr blt() {
    return waveguide::vague::builtins::get_instance();
}