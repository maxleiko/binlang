#include "binlang.h"

inline void bl_slice__advance(bl_slice_t *b, size_t n) {
  b->data += n;
  b->len += n;
}

bl_result_t bl_TODO(bl_unused bl_slice_t *b, bl_unused void *value) { return bl_result_err; }

bl_result_t bl_slice__read_u8(bl_slice_t *b, uint8_t *value) {
  if (b->len < 1) {
    return bl_result_eof;
  }
  *value = b->data[0];
  bl_slice__advance(b, 1);
  return bl_result_ok;
}

bl_result_t bl_slice__read_u16(bl_slice_t *b, uint16_t *value) {
  if (b->len < 2) {
    return bl_result_eof;
  }
  uint8_t *data = b->data;

  uint16_t tmp = 0;
  tmp |= (uint16_t)data[0] << 0;
  tmp |= (uint16_t)data[1] << 8;

  *value = tmp;
  bl_slice__advance(b, 2);
  return bl_result_ok;
}

bl_result_t bl_slice__read_u32(bl_slice_t *b, uint32_t *value) {
  if (b->len < 4) {
    return bl_result_eof;
  }
  uint8_t *data = b->data;

  uint32_t tmp = 0;
  tmp |= (uint32_t)data[0] << 0;
  tmp |= (uint32_t)data[1] << 8;
  tmp |= (uint32_t)data[2] << 16;
  tmp |= (uint32_t)data[3] << 24;

  *value = tmp;
  bl_slice__advance(b, 4);
  return bl_result_ok;
}

bl_result_t bl_slice__read_u64(bl_slice_t *b, uint64_t *value) {
  if (b->len < 8) {
    return bl_result_eof;
  }
  uint8_t *data = b->data;

  uint64_t tmp = 0;
  tmp |= (uint64_t)data[0] << 0;
  tmp |= (uint64_t)data[1] << 8;
  tmp |= (uint64_t)data[2] << 16;
  tmp |= (uint64_t)data[3] << 24;
  tmp |= (uint64_t)data[4] << 32;
  tmp |= (uint64_t)data[5] << 40;
  tmp |= (uint64_t)data[6] << 48;
  tmp |= (uint64_t)data[7] << 56;

  *value = tmp;
  bl_slice__advance(b, 8);
  return bl_result_ok;
}

bl_result_t bl_slice__read_i8(bl_slice_t *b, int8_t *value) {
  if (b->len < 1) {
    return bl_result_eof;
  }
  *value = (int8_t)b->data[0];
  bl_slice__advance(b, 1);
  return bl_result_ok;
}

bl_result_t bl_slice__read_i32(bl_slice_t *b, int32_t *value) {
  if (b->len < 4) {
    return bl_result_eof;
  }
  uint8_t *data = b->data;

  int32_t tmp = 0;
  tmp |= (int32_t)data[0] << 0;
  tmp |= (int32_t)data[1] << 8;
  tmp |= (int32_t)data[2] << 16;
  tmp |= (int32_t)data[3] << 24;

  *value = tmp;

  bl_slice__advance(b, 4);
  return bl_result_ok;
}

bl_result_t bl_slice__read_i64(bl_slice_t *b, int64_t *value) {
  if (b->len < 8) {
    return bl_result_eof;
  }

  uint8_t *data = b->data;

  int64_t tmp = 0;
  tmp |= (int64_t)data[0] << 0;
  tmp |= (int64_t)data[1] << 8;
  tmp |= (int64_t)data[2] << 16;
  tmp |= (int64_t)data[3] << 24;
  tmp |= (int64_t)data[4] << 32;
  tmp |= (int64_t)data[5] << 40;
  tmp |= (int64_t)data[6] << 48;
  tmp |= (int64_t)data[7] << 56;

  *value = tmp;

  bl_slice__advance(b, 8);
  return bl_result_ok;
}

bl_result_t bl_slice__read_vu32(bl_slice_t *b, uint32_t *value) {
  uint32_t result = 0;
  uint32_t shift = 0;
  size_t i = 0;

  while (i < b->len && i < 5) {
    uint8_t byte = b->data[i];
    result |= (uint32_t)(byte & 0x7F) << shift;
    shift += 7;
    i += 1;
    if ((byte & 0x80) == 0) {
      *value = result;
      bl_slice__advance(b, i);
      return bl_result_ok;
    }
  }

  return bl_result_eof;
}

bl_result_t bl_slice__read_vu64(bl_slice_t *b, uint64_t *value) {
  uint64_t result = 0;
  uint32_t shift = 0;
  size_t i = 0;

  while (i < b->len && i < 10) {
    uint8_t byte = b->data[i];
    result |= (uint64_t)(byte & 0x7F) << shift;
    shift += 7;
    i += 1;
    if ((byte & 0x80) == 0) {
      *value = result;
      bl_slice__advance(b, i);
      return bl_result_ok;
    }
  }

  return bl_result_eof;
}

bl_result_t bl_slice__read_vi32(bl_slice_t *b, int32_t *value) {
  int32_t result = 0;
  uint32_t shift = 0;
  size_t i = 0;

  while (i < b->len && i < 5) {
    uint8_t byte = b->data[i];
    result |= (int32_t)(byte & 0x7F) << shift;
    shift += 7;
    i += 1;
    if ((byte & 0x80) == 0) {
      if (shift < 32 && (byte & 0x40)) {
        result |= -(1 << shift);
      }
      *value = result;
      bl_slice__advance(b, i);
      return bl_result_ok;
    }
  }

  return bl_result_eof;
}

bl_result_t bl_slice__read_vi64(bl_slice_t *b, int64_t *value) {
  int64_t result = 0;
  uint32_t shift = 0;
  size_t i = 0;

  while (i < b->len && i < 10) {
    uint8_t byte = b->data[i];
    result |= (int64_t)(byte & 0x7F) << shift;
    shift += 7;
    i += 1;
    if ((byte & 0x80) == 0) {
      if (shift < 64 && (byte & 0x40)) {
        result |= -((int64_t)1 << shift);
      }
      *value = result;
      bl_slice__advance(b, i);
      return bl_result_ok;
    }
  }

  return bl_result_eof;
}