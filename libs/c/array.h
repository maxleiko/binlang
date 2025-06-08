#ifndef BINLANG_ARRAY_H_
#define BINLANG_ARRAY_H_

#ifdef __cplusplus
extern "C" {
#endif

#include "alloc.h"

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#ifdef _MSC_VER
#pragma warning(push)
#pragma warning(disable : 4101)
#elif defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-variable"
#endif

#define BlArray(T)                                                             \
  struct {                                                                     \
    T *elems;                                                                  \
    uint32_t size;                                                             \
    uint32_t capacity;                                                         \
  }

/// Initialize an array.
#define array_init(self)                                                       \
  ((self)->size = 0, (self)->capacity = 0, (self)->elems = NULL)

/// Create an empty array.
#define array_new()                                                            \
  { NULL, 0, 0 }

/// Get a pointer to the element at a given `index` in the array.
#define array_get(self, _index)                                                \
  (bl_assert((uint32_t)(_index) < (self)->size), &(self)->elems[_index])

/// Get a pointer to the first element in the array.
#define array_front(self) array_get(self, 0)

/// Get a pointer to the last element in the array.
#define array_back(self) array_get(self, (self)->size - 1)

/// Clear the array, setting its size to zero. Note that this does not free any
/// memory allocated for the array's elems.
#define array_clear(self) ((self)->size = 0)

/// Reserve `new_capacity` elements of space in the array. If `new_capacity` is
/// less than the array's current capacity, this function has no effect.
#define array_reserve(self, new_capacity)                                      \
  _array__reserve((BlArray *)(self), array_elem_size(self), new_capacity)

/// Free any memory allocated for this array. Note that this does not free any
/// memory allocated for the array's elems.
#define array_delete(self) _array__delete((BlArray *)(self))

/// Push a new `element` onto the end of the array.
#define array_push(self, element)                                              \
  (_array__grow((BlArray *)(self), 1, array_elem_size(self)),                  \
   (self)->elems[(self)->size++] = (element))

/// Increase the array's size by `count` elements.
/// New elements are zero-initialized.
#define array_grow_by(self, count)                                             \
  do {                                                                         \
    if ((count) == 0)                                                          \
      break;                                                                   \
    _array__grow((BlArray *)(self), count, array_elem_size(self));             \
    memset((self)->elems + (self)->size, 0, (count)*array_elem_size(self));    \
    (self)->size += (count);                                                   \
  } while (0)

/// Append all elements from one array to the end of another.
#define array_push_all(self, other)                                            \
  array_extend((self), (other)->size, (other)->elems)

/// Append `count` elements to the end of the array, reading their values from
/// the `elems` pointer.
#define array_extend(self, count, elems)                                       \
  _array__splice((BlArray *)(self), array_elem_size(self), (self)->size, 0,    \
                 count, elems)

/// Remove `old_count` elements from the array starting at the given `index`. At
/// the same index, insert `new_count` new elements, reading their values from
/// the `new_elems` pointer.
#define array_splice(self, _index, old_count, new_count, new_elems)            \
  _array__splice((BlArray *)(self), array_elem_size(self), _index, old_count,  \
                 new_count, new_elems)

/// Insert one `element` into the array at the given `index`.
#define array_insert(self, _index, element)                                    \
  _array__splice((BlArray *)(self), array_elem_size(self), _index, 0, 1,       \
                 &(element))

/// Remove one element from the array at the given `index`.
#define array_erase(self, _index)                                              \
  _array__erase((BlArray *)(self), array_elem_size(self), _index)

/// Pop the last element off the array, returning the element by value.
#define array_pop(self) ((self)->elems[--(self)->size])

/// Assign the elems of one array to another, reallocating if necessary.
#define array_assign(self, other)                                              \
  _array__assign((BlArray *)(self), (const BlArray *)(other),                  \
                 array_elem_size(self))

/// Swap one array with another
#define array_swap(self, other)                                                \
  _array__swap((BlArray *)(self), (BlArray *)(other))

/// Get the size of the array elems
#define array_elem_size(self) (sizeof *(self)->elems)

/// Search a sorted array for a given `needle` value, using the given `compare`
/// callback to determine the order.
///
/// If an existing element is found to be equal to `needle`, then the `index`
/// out-parameter is set to the existing value's index, and the `exists`
/// out-parameter is set to true. Otherwise, `index` is set to an index where
/// `needle` should be inserted in order to preserve the sorting, and `exists`
/// is set to false.
#define array_search_sorted_with(self, compare, needle, _index, _exists)       \
  _array__search_sorted(self, 0, compare, , needle, _index, _exists)

/// Search a sorted array for a given `needle` value, using integer comparisons
/// of a given struct field (specified with a leading dot) to determine the
/// order.
///
/// See also `array_search_sorted_with`.
#define array_search_sorted_by(self, field, needle, _index, _exists)           \
  _array__search_sorted(self, 0, _compare_int, field, needle, _index, _exists)

/// Insert a given `value` into a sorted array, using the given `compare`
/// callback to determine the order.
#define array_insert_sorted_with(self, compare, value)                         \
  do {                                                                         \
    unsigned _index, _exists;                                                  \
    array_search_sorted_with(self, compare, &(value), &_index, &_exists);      \
    if (!_exists)                                                              \
      array_insert(self, _index, value);                                       \
  } while (0)

/// Insert a given `value` into a sorted array, using integer comparisons of
/// a given struct field (specified with a leading dot) to determine the order.
///
/// See also `array_search_sorted_by`.
#define array_insert_sorted_by(self, field, value)                             \
  do {                                                                         \
    unsigned _index, _exists;                                                  \
    array_search_sorted_by(self, field, (value)field, &_index, &_exists);      \
    if (!_exists)                                                              \
      array_insert(self, _index, value);                                       \
  } while (0)

// Private

typedef BlArray(void) BlArray;

/// This is not what you're looking for, see `array_delete`.
static inline void _array__delete(BlArray *self) {
  if (self->elems) {
    bl_free(self->elems);
    self->elems = NULL;
    self->size = 0;
    self->capacity = 0;
  }
}

/// This is not what you're looking for, see `array_erase`.
static inline void _array__erase(BlArray *self, size_t element_size,
                                 uint32_t index) {
  bl_assert(index < self->size);
  char *elems = (char *)self->elems;
  memmove(elems + index * element_size, elems + (index + 1) * element_size,
          (self->size - index - 1) * element_size);
  self->size--;
}

/// This is not what you're looking for, see `array_reserve`.
static inline void _array__reserve(BlArray *self, size_t element_size,
                                   uint32_t new_capacity) {
  if (new_capacity > self->capacity) {
    if (self->elems) {
      self->elems = bl_realloc(self->elems, new_capacity * element_size);
    } else {
      self->elems = bl_malloc(new_capacity * element_size);
    }
    self->capacity = new_capacity;
  }
}

/// This is not what you're looking for, see `array_assign`.
static inline void _array__assign(BlArray *self, const BlArray *other,
                                  size_t element_size) {
  _array__reserve(self, element_size, other->size);
  self->size = other->size;
  memcpy(self->elems, other->elems, self->size * element_size);
}

/// This is not what you're looking for, see `array_swap`.
static inline void _array__swap(BlArray *self, BlArray *other) {
  BlArray swap = *other;
  *other = *self;
  *self = swap;
}

/// This is not what you're looking for, see `array_push` or `array_grow_by`.
static inline void _array__grow(BlArray *self, uint32_t count,
                                size_t element_size) {
  uint32_t new_size = self->size + count;
  if (new_size > self->capacity) {
    uint32_t new_capacity = self->capacity * 2;
    if (new_capacity < 8)
      new_capacity = 8;
    if (new_capacity < new_size)
      new_capacity = new_size;
    _array__reserve(self, element_size, new_capacity);
  }
}

/// This is not what you're looking for, see `array_splice`.
static inline void _array__splice(BlArray *self, size_t element_size,
                                  uint32_t index, uint32_t old_count,
                                  uint32_t new_count, const void *elements) {
  uint32_t new_size = self->size + new_count - old_count;
  uint32_t old_end = index + old_count;
  uint32_t new_end = index + new_count;
  bl_assert(old_end <= self->size);

  _array__reserve(self, element_size, new_size);

  char *elems = (char *)self->elems;
  if (self->size > old_end) {
    memmove(elems + new_end * element_size, elems + old_end * element_size,
            (self->size - old_end) * element_size);
  }
  if (new_count > 0) {
    if (elements) {
      memcpy((elems + index * element_size), elements,
             new_count * element_size);
    } else {
      memset((elems + index * element_size), 0, new_count * element_size);
    }
  }
  self->size += new_count - old_count;
}

/// A binary search routine, based on Rust's `std::slice::binary_search_by`.
/// This is not what you're looking for, see `array_search_sorted_with` or
/// `array_search_sorted_by`.
#define _array__search_sorted(self, start, compare, suffix, needle, _index,    \
                              _exists)                                         \
  do {                                                                         \
    *(_index) = start;                                                         \
    *(_exists) = false;                                                        \
    uint32_t size = (self)->size - *(_index);                                  \
    if (size == 0)                                                             \
      break;                                                                   \
    int comparison;                                                            \
    while (size > 1) {                                                         \
      uint32_t half_size = size / 2;                                           \
      uint32_t mid_index = *(_index) + half_size;                              \
      comparison = compare(&((self)->elems[mid_index] suffix), (needle));      \
      if (comparison <= 0)                                                     \
        *(_index) = mid_index;                                                 \
      size -= half_size;                                                       \
    }                                                                          \
    comparison = compare(&((self)->elems[*(_index)] suffix), (needle));        \
    if (comparison == 0)                                                       \
      *(_exists) = true;                                                       \
    else if (comparison < 0)                                                   \
      *(_index) += 1;                                                          \
  } while (0)

/// Helper macro for the `_sorted_by` routines below. This takes the left
/// (existing) parameter by reference in order to work with the generic sorting
/// function above.
#define _compare_int(a, b) ((int)*(a) - (int)(b))

#ifdef _MSC_VER
#pragma warning(pop)
#elif defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic pop
#endif

#ifdef __cplusplus
}
#endif

#endif // BINLANG_ARRAY_H_