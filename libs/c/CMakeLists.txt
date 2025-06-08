project(binlang C)

add_compile_definitions(
    $<$<OR:$<CONFIG:Release>,$<CONFIG:RelWithDebInfo>>:NDEBUG>
)

# Create the static lib
add_library(binlang STATIC
  alloc.c
  binlang.c
)

target_include_directories(binlang PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_compile_options(binlang PRIVATE -Wall -Wextra -Werror)