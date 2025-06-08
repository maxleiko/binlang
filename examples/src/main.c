#include <fcntl.h>
#include <stdio.h>
#include <unistd.h>

#include "binlang.h"
#include "greycat_abi.h"

typedef BlVec(uint8_t) byte_vec_t;

int read_file(const char *filepath, byte_vec_t *buf) {
  /* open the file read-only */
  int fd = open(filepath, O_RDONLY);
  if (fd < 0) {
    perror("open");
    return 1;
  }

  while (true) {
    vec_reserve(buf, buf->capacity + 8192);
    ssize_t n = read(fd, buf->elems + buf->size, buf->capacity - buf->size);
    if (n < 0) {
      perror("read");
      return 1;
    }
    if (n == 0) {
      return 0;
    }
    buf->size += n;
  }

  return 0;
}

int32_t main(int32_t argc, char *argv[]) {
  // check that a filepath is provided
  if (argc < 2) {
    fprintf(stderr, "Usage: %s <filepath>\n\n", argv[1]);
    fprintf(stderr, "  eg. %s gcdata/abi\n", argv[1]);
    return 1;
  }

  byte_vec_t buf = vec_new();
  if (read_file(argv[1], &buf) > 0) {
    return 1;
  }

  bl_slice_t b = {.data = buf.elems, .len = buf.size};
  abi_t abi = {0};
  if (bl_greycat_abi__read_abi(&b, &abi) <= 0) {
    fprintf(stderr, "unable to deserialize file\n");
    return 1;
  }

  printf("=== headers ===\n");
  printf("major=%d\n", abi.headers.major);
  printf("version=%d\n", abi.headers.version);
  printf("magic=%d\n", abi.headers.magic);
  printf("crc=%ld\n", abi.headers.crc);

  printf("=== symbols ===\n");
  for (uint32_t i = 0; i < abi.symbols.symbols.size; i++) {
    symbol_t *symbol = array_get(&abi.symbols.symbols, i);
    printf("%.*s=%d\n", symbol->text.size, symbol->text.elems, i);
  }

  printf("=== types ===\n");
  for (uint32_t i = 0; i < abi.types.types.size; i++) {
    type_t *ty = array_get(&abi.types.types, i);
    symbol_t *name = array_get(&abi.symbols.symbols, ty->name - 1);
    printf("%.*s=%d\n", name->text.size, name->text.elems, i);
  }

  printf("=== functions ===\n");
  for (uint32_t i = 0; i < abi.functions.functions.size; i++) {
    function_t *fn = array_get(&abi.functions.functions, i);
    symbol_t *name = array_get(&abi.symbols.symbols, fn->name - 1);
    printf("%.*s=%d\n", name->text.size, name->text.elems, i);
  }

  vec_delete(&buf);

  return 0;
}