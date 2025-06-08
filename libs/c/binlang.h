#ifndef binlang_h
#define binlang_h

#define bl_unused __attribute__((unused))

#ifdef FLOAT
#include <float.h>
typedef float f32_t;
typedef double f64_t;
#endif

#include "alloc.h"
#include "array.h"
#include "vec.h"

typedef struct {
  uint8_t *data;
  uint32_t len;
} bl_slice_t;

typedef enum {
  bl_result_err = -1,
  bl_result_eof = 0,
  bl_result_ok = 1,
} bl_result_t;

#define BL_TRY(x)                                                              \
  do {                                                                         \
    bl_result_t res = (x);                                                     \
    if (res <= 0) {                                                            \
      return res;                                                              \
    }                                                                          \
  } while (0);

void bl_slice__advance(bl_slice_t *b, size_t n);
/// Reads an unsigned 8-bit
bl_result_t bl_slice__read_u8(bl_slice_t *b, uint8_t *value);
/// Reads an unsigned 16-bit (little endian)
bl_result_t bl_slice__read_u16(bl_slice_t *b, uint16_t *value);
/// Reads an unsigned 32-bit (little endian)
bl_result_t bl_slice__read_u32(bl_slice_t *b, uint32_t *value);
/// Reads an unsigned 64-bit (little endian)
bl_result_t bl_slice__read_u64(bl_slice_t *b, uint64_t *value);
/// Reads a signed 8-bit
bl_result_t bl_slice__read_i8(bl_slice_t *b, int8_t *value);
/// Reads a signed 32-bit (little endian)
bl_result_t bl_slice__read_i32(bl_slice_t *b, int32_t *value);
/// Reads a signed 64-bit (little endian)
bl_result_t bl_slice__read_i64(bl_slice_t *b, int64_t *value);
/// Reads a LEB128-encoded unsigned 32-bit
bl_result_t bl_slice__read_vu32(bl_slice_t *b, uint32_t *value);
/// Reads a LEB128-encoded unsigned 64-bit
bl_result_t bl_slice__read_vu64(bl_slice_t *b, uint64_t *value);
/// Reads a LEB128-encoded signed 32-bit
bl_result_t bl_slice__read_vi32(bl_slice_t *b, int32_t *value);
/// Reads a LEB128-encoded signed 64-bit
bl_result_t bl_slice__read_vi64(bl_slice_t *b, int64_t *value);
/// Copies exactly `len` bytes from `b` into `buf`
bl_result_t bl_slice__read_exact(bl_slice_t *b, uint8_t *buf, uint64_t len);
#ifdef FLOAT
/// Reads a 32-bit floating-point number (little endian)
bl_result_t bl_slice__read_f32(bl_slice_t *b, f32_t *value);
/// Reads a 64-bit floating-point number (little endian)
bl_result_t bl_slice__read_f64(bl_slice_t *b, f64_t *value);
#endif

#endif // binlang_h