#include "binlang.h"

typedef struct Headers headers_t;
typedef struct Symbols symbols_t;
typedef struct Types types_t;
typedef struct Functions functions_t;
typedef struct FnParam fn_param_t;
typedef struct Abi abi_t;
typedef struct Function function_t;

struct Abi {
  headers_t headers;
  symbols_t symbols;
  types_t types;
  functions_t functions;
};

struct Headers {
  uint16_t major;
  uint16_t magic;
  uint16_t version;
  uint64_t crc;
};

struct Symbols {
  uint64_t byte_size;
  BlArray(symbol_t) symbols;
};

struct Symbol {
  uint32_t size;
  uint8_t text;
};

struct Types {
  uint64_t byte_size;
  uint32_t nb_types;
  uint32_t nb_attrs;
  type_t types;
};

struct Type {
  uint32_t module;
  uint32_t name;
  uint32_t lib;
  uint32_t generic_abi_type;
  uint32_t g1;
  uint32_t g2;
  uint32_t super_type;
  uint32_t nb_attrs;
  uint32_t attrs_off;
  uint32_t mapped_prog_type_off;
  uint32_t masked_abi_type_off;
  uint32_t nullable_nb_bytes;
  type_flags_t flags;
  type_attr_t attrs;
};

// Bitfield: TypeFlags
#define TYPEFLAGS_NATIVE (1 << 0)
#define TYPEFLAGS_ABSTRACT (1 << 1)
#define TYPEFLAGS_ENUM (1 << 2)
#define TYPEFLAGS_MASKED (1 << 3)
#define TYPEFLAGS_AMBIGUOUS (1 << 4)

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

// Bitfield: TypeAttrFlags
#define TYPEATTRFLAGS_NULLABLE (1 << 0)
#define TYPEATTRFLAGS_MAPPED (1 << 1)

struct Functions {
  // @done_if_zero
  uint64_t byte_size;
  uint32_t nb_functions;
  function_t functions;
};

struct Function {
  uint32_t module;
  uint32_t type;
  uint32_t name;
  uint32_t lib;
  uint32_t arity;
  fn_param_t params;
  uint32_t return_type;
  fn_param_t flags;
};

// Bitfield: FunctionFlags
#define FUNCTIONFLAGS_RETURN_NULLABLE (1 << 0)

struct FnParam {
  uint8_t nullable;
  uint32_t type;
  uint32_t name;
};


