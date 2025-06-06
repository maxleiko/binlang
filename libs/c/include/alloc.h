#ifndef BINLANG_ALLOC_H_
#define BINLANG_ALLOC_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

extern void *(*bl_current_malloc)(size_t size);
extern void *(*bl_current_calloc)(size_t count, size_t size);
extern void *(*bl_current_realloc)(void *ptr, size_t size);
extern void (*bl_current_free)(void *ptr);

// Allow clients to override allocation functions
#ifndef bl_malloc
#define bl_malloc  bl_current_malloc
#endif
#ifndef bl_calloc
#define bl_calloc  bl_current_calloc
#endif
#ifndef bl_realloc
#define bl_realloc bl_current_realloc
#endif
#ifndef bl_free
#define bl_free    bl_current_free
#endif

#ifdef __cplusplus
}
#endif

#endif // BINLANG_ALLOC_H_