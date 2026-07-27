// Test header for kitc-ffi end-to-end testing
// This header contains various C declarations that should be extracted correctly.

#ifndef TEST_HEADER_H
#define TEST_HEADER_H

#include <stddef.h>
#include <stdint.h>

// Simple function prototypes
int add(int a, int b);
double multiply(double x, double y);
void greet(const char *name);
void *allocate(size_t size);

// Variadic function
int printf(const char *format, ...);

// Function with no parameters
int get_value(void);

// Struct definition
struct Point {
    int x;
    int y;
};

// Struct with pointer fields
struct Node {
    int value;
    struct Node *next;
};

// Typedef of a struct
typedef struct {
    int width;
    int height;
} Rectangle;

// Simple typedef
typedef unsigned long ulong;

// Enum definition
enum Color {
    RED,
    GREEN,
    BLUE = 5
};

// Typedef enum
typedef enum {
    MONDAY = 1,
    TUESDAY,
    WEDNESDAY
} Weekday;

// Function taking struct by pointer
double distance(struct Point *a, struct Point *b);

// Function returning struct
struct Point make_point(int x, int y);

// Callback typedef
typedef void (*callback_t)(int value);

// Function with callback parameter
void set_callback(callback_t cb);

// Const pointer parameter
size_t string_length(const char *str);

// Array parameter
int sum_array(int arr[], int count);

// Global variable declarations
extern int global_counter;
extern const char *version_string;

#endif // TEST_HEADER_H