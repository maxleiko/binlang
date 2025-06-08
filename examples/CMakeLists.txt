cmake_minimum_required(VERSION 3.6)
project(greycat_abi C)

set(PROJECT_SOURCES
  src/main.c
  src/greycat_abi.c
)

add_subdirectory(../libs/c ${CMAKE_BINARY_DIR}/binlang)

add_executable(main ${PROJECT_SOURCES} ${BINLANG_SOURCES})
target_link_libraries(main PRIVATE binlang)
target_include_directories(main PRIVATE ${CMAKE_SOURCE_DIR}/../libs/c)

target_compile_options(main PRIVATE -Wall -Wextra -Werror)
# Whether or not to generate read/write functions and types for floats (requires float.h)
target_compile_options(main PRIVATE -DFLOAT)