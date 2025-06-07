#include "binlang.h"

typedef struct Bar bar_t;
typedef struct Foo foo_t;
typedef struct Temperatures temperatures_t;
typedef struct Sensor sensor_t;


struct Bar {
  BlArray(uint8_t) data;
  uint32_t extra;
};

struct Foo {
};

struct Temperatures {
  BlArray(sensor_t) sensors;
};

struct Sensor {
  uint32_t temp;
  uint32_t pressure;
  uint32_t nb;
  BlArray(foo_t) foos;
  bar_t bar;
};


