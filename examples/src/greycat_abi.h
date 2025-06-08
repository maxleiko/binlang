#ifndef BINLANG_greycat_abi_H_
#define BINLANG_greycat_abi_H_

#include "binlang.h"

typedef struct FnParam fn_param_t;
typedef uint8_t function_flags_t;
typedef struct Function function_t;
typedef struct Functions functions_t;
typedef uint8_t type_attr_flags_t;
typedef struct TypeAttr type_attr_t;
typedef uint8_t type_flags_t;
typedef struct Type type_t;
typedef struct Types types_t;
typedef struct Symbol symbol_t;
typedef struct Symbols symbols_t;
typedef struct Headers headers_t;
typedef struct Abi abi_t;

/// Bitfield: FunctionFlags
#define FUNCTION_FLAGS_RETURN_NULLABLE (1 << 0)

/// Bitfield: TypeAttrFlags
#define TYPE_ATTR_FLAGS_NULLABLE (1 << 0)
#define TYPE_ATTR_FLAGS_MAPPED (1 << 1)

/// Bitfield: TypeFlags
#define TYPE_FLAGS_NATIVE (1 << 0)
#define TYPE_FLAGS_ABSTRACT (1 << 1)
#define TYPE_FLAGS_ENUM (1 << 2)
#define TYPE_FLAGS_MASKED (1 << 3)
#define TYPE_FLAGS_AMBIGUOUS (1 << 4)

struct FnParam {
  uint8_t nullable;
  uint32_t type;
  uint32_t name;
};

struct Function {
  uint32_t module;
  uint32_t type;
  uint32_t name;
  uint32_t lib;
  BlArray(fn_param_t) params;
  uint32_t return_type;
  function_flags_t flags;
};

struct Functions {
  uint64_t byte_size;
  BlArray(function_t) functions;
};

struct TypeAttr {
  uint32_t name;
  uint32_t abi_type;
  uint32_t prog_type_off;
  uint32_t mapped_any_off;
  uint32_t mapped_att_off;
  uint8_t sbi_type;
  uint8_t precision;
  type_attr_flags_t flags;
};

struct Type {
  uint32_t module;
  uint32_t name;
  uint32_t lib;
  uint32_t generic_abi_type;
  uint32_t g1;
  uint32_t g2;
  uint32_t super_type;
  uint32_t attrs_off;
  uint32_t mapped_prog_type_off;
  uint32_t mapped_abi_type_off;
  uint32_t masked_abi_type_off;
  uint32_t nullable_nb_bytes;
  type_flags_t flags;
  BlArray(type_attr_t) attrs;
};

struct Types {
  uint64_t byte_size;
  uint32_t nb_attrs;
  BlArray(type_t) types;
};

struct Symbol {
  BlArray(uint8_t) text;
};

struct Symbols {
  uint64_t byte_size;
  BlArray(symbol_t) symbols;
};

struct Headers {
  uint16_t major;
  uint16_t magic;
  uint32_t version;
  uint64_t crc;
};

struct Abi {
  headers_t headers;
  symbols_t symbols;
  types_t types;
  functions_t functions;
};

bl_result_t bl_greycat_abi__read_fn_param(bl_slice_t *b, fn_param_t *value);
bl_result_t bl_greycat_abi__read_function(bl_slice_t *b, function_t *value);
bl_result_t bl_greycat_abi__read_functions(bl_slice_t *b, functions_t *value);
bl_result_t bl_greycat_abi__read_type_attr(bl_slice_t *b, type_attr_t *value);
bl_result_t bl_greycat_abi__read_type(bl_slice_t *b, type_t *value);
bl_result_t bl_greycat_abi__read_types(bl_slice_t *b, types_t *value);
bl_result_t bl_greycat_abi__read_symbol(bl_slice_t *b, symbol_t *value);
bl_result_t bl_greycat_abi__read_symbols(bl_slice_t *b, symbols_t *value);
bl_result_t bl_greycat_abi__read_headers(bl_slice_t *b, headers_t *value);
bl_result_t bl_greycat_abi__read_abi(bl_slice_t *b, abi_t *value);

#endif // BINLANG_greycat_abi_H_
