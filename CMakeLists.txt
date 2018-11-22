cmake_minimum_required(VERSION 3.10.0)
project(Waveguide)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_BUILD_TYPE Release)
set(SOURCE_DIR source/x3test)
set(SPIRIT_DIR spirit)

include_directories(${SOURCE_DIR})
include_directories(${SPIRIT_DIR}/include)
file(GLOB_RECURSE SRC_FILES ${SOURCE_DIR}/*.cpp)

add_executable(waveguide ${SRC_FILES})
target_link_libraries(waveguide boost_system)
