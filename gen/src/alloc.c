#include "alloc.h"
#include <stdlib.h>

static void *bl_malloc_default(size_t size) {
  void *result = malloc(size);
  if (size > 0 && !result) {
    fprintf(stderr, "binlang failed to allocate %zu bytes", size);
    abort();
  }
  return result;
}

static void *bl_calloc_default(size_t count, size_t size) {
  void *result = calloc(count, size);
  if (count > 0 && !result) {
    fprintf(stderr, "binlang failed to allocate %zu bytes", count * size);
    abort();
  }
  return result;
}

static void *bl_realloc_default(void *buffer, size_t size) {
  void *result = realloc(buffer, size);
  if (size > 0 && !result) {
    fprintf(stderr, "binlang failed to reallocate %zu bytes", size);
    abort();
  }
  return result;
}

// Allow clients to override allocation functions dynamically
void *(*bl_current_malloc)(size_t) = bl_malloc_default;
void *(*bl_current_calloc)(size_t, size_t) = bl_calloc_default;
void *(*bl_current_realloc)(void *, size_t) = bl_realloc_default;
void (*bl_current_free)(void *) = free;

void bl_set_allocator(void *(*new_malloc)(size_t size),
                      void *(*new_calloc)(size_t count, size_t size),
                      void *(*new_realloc)(void *ptr, size_t size),
                      void (*new_free)(void *ptr)) {
  bl_current_malloc = new_malloc ? new_malloc : bl_malloc_default;
  bl_current_calloc = new_calloc ? new_calloc : bl_calloc_default;
  bl_current_realloc = new_realloc ? new_realloc : bl_realloc_default;
  bl_current_free = new_free ? new_free : free;
}