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
    fprintf(stderr, "Usage: %s <file>\n", argv[1]);
    return 1;
  }

  byte_vec_t buf = vec_new();
  if (read_file(argv[1], &buf) > 0) {
    return 1;
  }

  bl_slice_t b = {.data = buf.elems, .len = buf.size};
  abi_t abi = {0};
  bl_result_t res = bl_greycat_abi__read_abi(&b, &abi);
  printf("res=%d\n", res);

  vec_delete(&buf);

  return 0;
}