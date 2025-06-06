#ifndef BINLANG_ASSERT_H_
#define BINLANG_ASSERT_H_

#ifdef NDEBUG
#define bl_assert(e) ((void)(e))
#else
#include <assert.h>
#define bl_assert(e) assert(e)
#endif

#endif // BINLANG_ASSERT_H_