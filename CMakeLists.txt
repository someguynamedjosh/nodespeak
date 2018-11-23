cmake_minimum_required(VERSION 3.10.0)
project(Waveguide)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_BUILD_TYPE Release)
set(SOURCE_DIR source/x3test)
set(SPIRIT_DIR spirit)

set(STANDALONE_TARGET waveguide_standalone)
set(STATIC_LIB_TARGET waveguide_a)

if(MSVC)
  add_compile_options(/W4 /WX)
else(MSVC)
  add_compile_options(-Wall -Wextra -pedantic)
endif(MSVC)

include_directories(${SOURCE_DIR})
include_directories(${SPIRIT_DIR}/include)
file(GLOB_RECURSE SRC_FILES ${SOURCE_DIR}/*.cpp)

add_executable(${STANDALONE_TARGET} ${SRC_FILES})
target_link_libraries(${STANDALONE_TARGET} boost_system)
