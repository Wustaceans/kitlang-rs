use std::collections::HashMap;

/// Provides minimal fake versions of common C system headers.
///
/// This allows parsing headers that include system headers without requiring
/// the actual system include paths.
pub struct FakeHeaders;

impl FakeHeaders {
    /// Returns true if the given header name has a fake version available.
    pub fn has_fake(name: &str) -> bool {
        SYSTEM_HEADERS.contains_key(name)
    }

    /// Returns the content of a fake system header, or None if not available.
    pub fn get(name: &str) -> Option<&'static str> {
        SYSTEM_HEADERS.get(name).copied()
    }

    /// Returns all available fake header names.
    pub fn available() -> Vec<&'static str> {
        SYSTEM_HEADERS.keys().copied().collect()
    }
}

use std::sync::LazyLock;

static SYSTEM_HEADERS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("stddef.h", STDDEF_H);
    m.insert("stdint.h", STDINT_H);
    m.insert("stdbool.h", STDBOOL_H);
    m.insert("stdarg.h", STDARG_H);
    m.insert("limits.h", LIMITS_H);
    m.insert("float.h", FLOAT_H);
    m.insert("stdlib.h", STDLIB_H);
    m.insert("stdio.h", STDIO_H);
    m.insert("string.h", STRING_H);
    m.insert("errno.h", ERRNO_H);
    m.insert("wchar.h", WCHAR_H);
    m.insert("wctype.h", WCTYPE_H);
    m.insert("time.h", TIME_H);
    m.insert("assert.h", ASSERT_H);
    m.insert("ctype.h", CTYPE_H);
    m.insert("signal.h", SIGNAL_H);
    m.insert("locale.h", LOCALE_H);
    m.insert("setjmp.h", SETJMP_H);
    m.insert("math.h", MATH_H);
    m.insert("inttypes.h", INTTYPES_H);
    m.insert("unistd.h", UNISTD_H);
    m.insert("sys/types.h", SYS_TYPES_H);
    m.insert("sys/stat.h", SYS_STAT_H);
    m.insert("fcntl.h", FCNTL_H);
    m.insert("pthread.h", PTHREAD_H);
    m.insert("dlfcn.h", DLFCN_H);
    m.insert("stdnoreturn.h", STDNORETURN_H);
    m.insert("stdalign.h", STDALIGN_H);
    m.insert("uchar.h", UCHAR_H);
    m.insert("complex.h", COMPLEX_H);
    m.insert("tgmath.h", TGMATH_H);
    m.insert("iso646.h", ISO646_H);
    m.insert("stdatomic.h", STDATOMIC_H);
    m.insert("threads.h", THREADS_H);
    m
});

static STDDEF_H: &str = r#"
#ifndef _STDDEF_H
#define _STDDEF_H

typedef __typeof__(sizeof(0)) size_t;
typedef __typeof__((char*)0 - (char*)0) ptrdiff_t;
typedef __typeof__(0) wchar_t;

#ifndef NULL
#define NULL ((void*)0)
#endif

#define offsetof(type, member) __builtin_offsetof(type, member)

#endif
"#;

static STDINT_H: &str = r#"
#ifndef _STDINT_H
#define _STDINT_H

typedef signed char int8_t;
typedef short int16_t;
typedef int int32_t;
typedef long long int64_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef signed char int_least8_t;
typedef short int_least16_t;
typedef int int_least32_t;
typedef long long int_least64_t;
typedef unsigned char uint_least8_t;
typedef unsigned short uint_least16_t;
typedef unsigned int uint_least32_t;
typedef unsigned long long uint_least64_t;

typedef int int_fast8_t;
typedef int int_fast16_t;
typedef int int_fast32_t;
typedef long long int_fast64_t;
typedef unsigned int uint_fast8_t;
typedef unsigned int uint_fast16_t;
typedef unsigned int uint_fast32_t;
typedef unsigned long long uint_fast64_t;

typedef long long intmax_t;
typedef unsigned long long uintmax_t;
typedef long intptr_t;
typedef unsigned long uintptr_t;

#define INT8_MIN (-128)
#define INT16_MIN (-32768)
#define INT32_MIN (-2147483648)
#define INT64_MIN (-9223372036854775808LL)

#define INT8_MAX 127
#define INT16_MAX 32767
#define INT32_MAX 2147483647
#define INT64_MAX 9223372036854775807LL

#define UINT8_MAX 255
#define UINT16_MAX 65535
#define UINT32_MAX 4294967295U
#define UINT64_MAX 18446744073709551615ULL

#define INT_LEAST8_MIN INT8_MIN
#define INT_LEAST16_MIN INT16_MIN
#define INT_LEAST32_MIN INT32_MIN
#define INT_LEAST64_MIN INT64_MIN
#define INT_LEAST8_MAX INT8_MAX
#define INT_LEAST16_MAX INT16_MAX
#define INT_LEAST32_MAX INT32_MAX
#define INT_LEAST64_MAX INT64_MAX
#define UINT_LEAST8_MAX UINT8_MAX
#define UINT_LEAST16_MAX UINT16_MAX
#define UINT_LEAST32_MAX UINT32_MAX
#define UINT_LEAST64_MAX UINT64_MAX

#define INT_FAST8_MIN INT32_MIN
#define INT_FAST16_MIN INT32_MIN
#define INT_FAST32_MIN INT32_MIN
#define INT_FAST64_MIN INT64_MIN
#define INT_FAST8_MAX INT32_MAX
#define INT_FAST16_MAX INT32_MAX
#define INT_FAST32_MAX INT32_MAX
#define INT_FAST64_MAX INT64_MAX
#define UINT_FAST8_MAX UINT32_MAX
#define UINT_FAST16_MAX UINT32_MAX
#define UINT_FAST32_MAX UINT32_MAX
#define UINT_FAST64_MAX UINT64_MAX

#define INTPTR_MIN LONG_MIN
#define INTPTR_MAX LONG_MAX
#define UINTPTR_MAX ULONG_MAX
#define INTMAX_MIN INT64_MIN
#define INTMAX_MAX INT64_MAX
#define UINTMAX_MAX UINT64_MAX

#define PTRDIFF_MIN (-__PTRDIFF_MAX__ - 1)
#define PTRDIFF_MAX __PTRDIFF_MAX__
#define SIZE_MAX __SIZE_MAX__
#define WCHAR_MIN __WCHAR_MIN__
#define WCHAR_MAX __WCHAR_MAX__

#define SIG_ATOMIC_MIN __SIG_ATOMIC_MIN__
#define SIG_ATOMIC_MAX __SIG_ATOMIC_MAX__

#define INT8_C(x) (x)
#define INT16_C(x) (x)
#define INT32_C(x) (x)
#define INT64_C(x) (x ## LL)
#define UINT8_C(x) (x)
#define UINT16_C(x) (x)
#define UINT32_C(x) (x ## U)
#define UINT64_C(x) (x ## ULL)
#define INTMAX_C(x) (x ## LL)
#define UINTMAX_C(x) (x ## ULL)

#endif
"#;

static STDBOOL_H: &str = r#"
#ifndef _STDBOOL_H
#define _STDBOOL_H

#define bool _Bool
#define true 1
#define false 0
#define __bool_true_false_are_defined 1

#endif
"#;

static STDARG_H: &str = r#"
#ifndef _STDARG_H
#define _STDARG_H

typedef __builtin_va_list va_list;

#define va_start(v, l) __builtin_va_start(v, l)
#define va_end(v) __builtin_va_end(v)
#define va_arg(v, l) __builtin_va_arg(v, l)
#define va_copy(d, s) __builtin_va_copy(d, s)

#endif
"#;

static LIMITS_H: &str = r#"
#ifndef _LIMITS_H
#define _LIMITS_H

#define CHAR_BIT 8
#define MB_LEN_MAX 16

#define CHAR_MIN (-128)
#define CHAR_MAX 127
#define SCHAR_MIN (-128)
#define SCHAR_MAX 127
#define UCHAR_MAX 255

#define SHRT_MIN (-32768)
#define SHRT_MAX 32767
#define USHRT_MAX 65535

#define INT_MIN (-2147483648)
#define INT_MAX 2147483647
#define UINT_MAX 4294967295U

#define LONG_MIN (-9223372036854775808L)
#define LONG_MAX 9223372036854775807L
#define ULONG_MAX 18446744073709551615UL

#define LLONG_MIN (-9223372036854775808LL)
#define LLONG_MAX 9223372036854775807LL
#define ULLONG_MAX 18446744073709551615ULL

#endif
"#;

static FLOAT_H: &str = r#"
#ifndef _FLOAT_H
#define _FLOAT_H

#define FLT_RADIX 2
#define FLT_ROUNDS 1
#define FLT_DIG 6
#define FLT_EPSILON 1.19209290e-07F
#define FLT_MANT_DIG 24
#define FLT_MAX 3.40282347e+38F
#define FLT_MAX_EXP 128
#define FLT_MAX_10_EXP 38
#define FLT_MIN 1.17549435e-38F
#define FLT_MIN_EXP (-125)
#define FLT_MIN_10_EXP (-37)

#define DBL_DIG 15
#define DBL_EPSILON 2.2204460492503131e-16
#define DBL_MANT_DIG 53
#define DBL_MAX 1.7976931348623157e+308
#define DBL_MAX_EXP 1024
#define DBL_MAX_10_EXP 308
#define DBL_MIN 2.2250738585072014e-308
#define DBL_MIN_EXP (-1021)
#define DBL_MIN_10_EXP (-307)

#endif
"#;

static STDLIB_H: &str = r#"
#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

#define EXIT_FAILURE 1
#define EXIT_SUCCESS 0
#define RAND_MAX 32767
#define MB_CUR_MAX 1

typedef struct { int quot; int rem; } div_t;
typedef struct { long quot; long rem; } ldiv_t;
typedef struct { long long quot; long long rem; } lldiv_t;

void *malloc(size_t size);
void *calloc(size_t nmemb, size_t size);
void *realloc(void *ptr, size_t size);
void free(void *ptr);
void abort(void);
void exit(int status);
int atexit(void (*func)(void));
int system(const char *command);
char *getenv(const char *name);
void *bsearch(const void *key, const void *base, size_t nmemb, size_t size, int (*compar)(const void *, const void *));
void qsort(void *base, size_t nmemb, size_t size, int (*compar)(const void *, const void *));
int abs(int x);
long labs(long x);
long long llabs(long long x);
div_t div(int numer, int denom);
ldiv_t ldiv(long numer, long denom);
lldiv_t lldiv(long long numer, long long denom);
int rand(void);
void srand(unsigned int seed);
int atoi(const char *nptr);
long atol(const char *nptr);
long long atoll(const char *nptr);
double atof(const char *nptr);
long strtol(const char *nptr, char **endptr, int base);
unsigned long strtoul(const char *nptr, char **endptr, int base);
long long strtoll(const char *nptr, char **endptr, int base);
unsigned long long strtoull(const char *nptr, char **endptr, int base);
double strtod(const char *nptr, char **endptr);
float strtof(const char *nptr, char **endptr);
long double strtold(const char *nptr, char **endptr);

#endif
"#;

static STDIO_H: &str = r#"
#ifndef _STDIO_H
#define _STDIO_H

#include <stddef.h>
#include <stdarg.h>

#define _IOFBF 0
#define _IOLBF 1
#define _IONBF 2

#define BUFSIZ 8192
#define EOF (-1)
#define FOPEN_MAX 16
#define FILENAME_MAX 4096
#define L_tmpnam 20
#define TMP_MAX 238328
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

typedef struct _IO_FILE FILE;
extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;

int remove(const char *pathname);
int rename(const char *old, const char *new);
FILE *tmpfile(void);
char *tmpnam(char *s);
int fclose(FILE *stream);
int fflush(FILE *stream);
FILE *fopen(const char *pathname, const char *mode);
FILE *freopen(const char *pathname, const char *mode, FILE *stream);
FILE *fdopen(int fd, const char *mode);
void setbuf(FILE *stream, char *buf);
int setvbuf(FILE *stream, char *buf, int mode, size_t size);
int fprintf(FILE *stream, const char *format, ...);
int fscanf(FILE *stream, const char *format, ...);
int printf(const char *format, ...);
int scanf(const char *format, ...);
int sprintf(char *str, const char *format, ...);
int sscanf(const char *str, const char *format, ...);
int snprintf(char *str, size_t size, const char *format, ...);
int vfprintf(FILE *stream, const char *format, va_list ap);
int vprintf(const char *format, va_list ap);
int vsprintf(char *str, const char *format, va_list ap);
int vsnprintf(char *str, size_t size, const char *format, va_list ap);
int fgetc(FILE *stream);
char *fgets(char *s, int size, FILE *stream);
int fputc(int c, FILE *stream);
int fputs(const char *s, FILE *stream);
int getc(FILE *stream);
int getchar(void);
char *gets(char *s);
int putc(int c, FILE *stream);
int putchar(int c);
int puts(const char *s);
int ungetc(int c, FILE *stream);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
void rewind(FILE *stream);
int fgetpos(FILE *stream, fpos_t *pos);
int fsetpos(FILE *stream, const fpos_t *pos);
void clearerr(FILE *stream);
int feof(FILE *stream);
int ferror(FILE *stream);
void perror(const char *s);

#endif
"#;

static STRING_H: &str = r#"
#ifndef _STRING_H
#define _STRING_H

#include <stddef.h>

void *memcpy(void *dest, const void *src, size_t n);
void *memmove(void *dest, const void *src, size_t n);
void *memchr(const void *s, int c, size_t n);
int memcmp(const void *s1, const void *s2, size_t n);
void *memset(void *s, int c, size_t n);
char *strcat(char *dest, const char *src);
char *strncat(char *dest, const char *src, size_t n);
char *strchr(const char *s, int c);
int strcmp(const char *s1, const char *s2);
int strncmp(const char *s1, const char *s2, size_t n);
char *strcpy(char *dest, const char *src);
char *strncpy(char *dest, const char *src, size_t n);
size_t strcspn(const char *s, const char *reject);
size_t strlen(const char *s);
char *strpbrk(const char *s, const char *accept);
char *strrchr(const char *s, int c);
size_t strspn(const char *s, const char *accept);
char *strstr(const char *haystack, const char *needle);
char *strtok(char *str, const char *delim);
char *strerror(int errnum);

#endif
"#;

static ERRNO_H: &str = r#"
#ifndef _ERRNO_H
#define _ERRNO_H

extern int errno;

#define EDOM 33
#define ERANGE 34
#define EILSEQ 84
#define EACCES 13
#define EAGAIN 11
#define EBADF 9
#define EEXIST 17
#define EFAULT 14
#define EINTR 4
#define EINVAL 22
#define EIO 5
#define EISDIR 21
#define EMFILE 24
#define ENAMETOOLONG 36
#define ENFILE 23
#define ENOENT 2
#define ENOMEM 12
#define ENOSPC 28
#define ENOTDIR 20
#define EPERM 1
#define EPIPE 32
#define ERANGE 34
#define ESPIPE 29
#define ESRCH 3
#define EXDEV 18

#endif
"#;

static WCHAR_H: &str = r#"
#ifndef _WCHAR_H
#define _WCHAR_H

#include <stddef.h>
#include <stdarg.h>
#include <stdio.h>

typedef struct {
    int __count;
    union {
        unsigned int __wch;
        char __wchb[4];
    } __value;
} mbstate_t;

typedef unsigned int wint_t;

#ifndef WEOF
#define WEOF ((wint_t)-1)
#endif

typedef struct {
    size_t __count;
    union {
        unsigned int __wch;
        char __wchb[4];
    } __value;
} mbstate_t;

wint_t btowc(int c);
int wctob(wint_t c);
size_t mbrtowc(wchar_t *pwc, const char *s, size_t n, mbstate_t *ps);
size_t wcrtomb(char *s, wchar_t wc, mbstate_t *ps);
size_t mbsrtowcs(wchar_t *dst, const char **src, size_t len, mbstate_t *ps);
size_t wcsrtombs(char *dst, const wchar_t **src, size_t len, mbstate_t *ps);
wchar_t *wcscpy(wchar_t *dest, const wchar_t *src);
wchar_t *wcsncpy(wchar_t *dest, const wchar_t *src, size_t n);
wchar_t *wcscat(wchar_t *dest, const wchar_t *src);
wchar_t *wcsncat(wchar_t *dest, const wchar_t *src, size_t n);
int wcscmp(const wchar_t *s1, const wchar_t *s2);
int wcsncmp(const wchar_t *s1, const wchar_t *s2, size_t n);
size_t wcslen(const wchar_t *s);
wchar_t *wcschr(const wchar_t *s, wchar_t c);
wchar_t *wcsrchr(const wchar_t *s, wchar_t c);
size_t wcsspn(const wchar_t *s, const wchar_t *accept);
size_t wcscspn(const wchar_t *s, const wchar_t *reject);
wchar_t *wcspbrk(const wchar_t *s, const wchar_t *accept);
wchar_t *wcsstr(const wchar_t *haystack, const wchar_t *needle);
wchar_t *wcstok(wchar_t *str, const wchar_t *delim, wchar_t **saveptr);
int fwide(FILE *stream, int mode);
wint_t fgetwc(FILE *stream);
wchar_t *fgetws(wchar_t *s, int n, FILE *stream);
wint_t fputwc(wchar_t c, FILE *stream);
int fputws(const wchar_t *s, FILE *stream);
wint_t ungetwc(wint_t c, FILE *stream);
wint_t getwc(FILE *stream);
wint_t getwchar(void);
wint_t putwc(wchar_t c, FILE *stream);
wint_t putwchar(wchar_t c);
int fwprintf(FILE *stream, const wchar_t *format, ...);
int wprintf(const wchar_t *format, ...);
int swprintf(wchar_t *s, size_t n, const wchar_t *format, ...);
int vfwprintf(FILE *stream, const wchar_t *format, va_list arg);
int vwprintf(const wchar_t *format, va_list arg);
int vswprintf(wchar_t *s, size_t n, const wchar_t *format, va_list arg);
int fwscanf(FILE *stream, const wchar_t *format, ...);
int wscanf(const wchar_t *format, ...);
int swscanf(const wchar_t *s, const wchar_t *format, ...);

#endif
"#;

static WCTYPE_H: &str = r#"
#ifndef _WCTYPE_H
#define _WCTYPE_H

#include <wchar.h>

typedef long wctype_t;
typedef const int *wctrans_t;

int iswalnum(wint_t wc);
int iswalpha(wint_t wc);
int iswblank(wint_t wc);
int iswcntrl(wint_t wc);
int iswdigit(wint_t wc);
int iswgraph(wint_t wc);
int iswlower(wint_t wc);
int iswprint(wint_t wc);
int iswpunct(wint_t wc);
int iswspace(wint_t wc);
int iswupper(wint_t wc);
int iswxdigit(wint_t wc);
wint_t towlower(wint_t wc);
wint_t towupper(wint_t wc);
wctype_t wctype(const char *property);
int iswctype(wint_t wc, wctype_t desc);
wctrans_t wctrans(const char *property);
wint_t towctrans(wint_t wc, wctrans_t desc);

#endif
"#;

static TIME_H: &str = r#"
#ifndef _TIME_H
#define _TIME_H

#include <stddef.h>

#define CLOCKS_PER_SEC 1000000L

typedef long clock_t;
typedef long time_t;

struct tm {
    int tm_sec;
    int tm_min;
    int tm_hour;
    int tm_mday;
    int tm_mon;
    int tm_year;
    int tm_wday;
    int tm_yday;
    int tm_isdst;
};

clock_t clock(void);
time_t time(time_t *t);
time_t mktime(struct tm *tm);
struct tm *gmtime(const time_t *t);
struct tm *localtime(const time_t *t);
char *asctime(const struct tm *tm);
char *ctime(const time_t *t);
double difftime(time_t time1, time_t time0);
size_t strftime(char *s, size_t max, const char *format, const struct tm *tm);

#endif
"#;

static ASSERT_H: &str = r#"
#ifndef _ASSERT_H
#define _ASSERT_H

void __assert_fail(const char *assertion, const char *file, unsigned int line, const char *function);

#ifdef NDEBUG
#define assert(expr) ((void)0)
#else
#define assert(expr) ((expr) ? (void)0 : __assert_fail(#expr, __FILE__, __LINE__, __func__))
#endif

#endif
"#;

static CTYPE_H: &str = r#"
#ifndef _CTYPE_H
#define _CTYPE_H

int isalnum(int c);
int isalpha(int c);
int isblank(int c);
int iscntrl(int c);
int isdigit(int c);
int isgraph(int c);
int islower(int c);
int isprint(int c);
int ispunct(int c);
int isspace(int c);
int isupper(int c);
int isxdigit(int c);
int tolower(int c);
int toupper(int c);

#endif
"#;

static SIGNAL_H: &str = r#"
#ifndef _SIGNAL_H
#define _SIGNAL_H

typedef int sig_atomic_t;
typedef void (*sighandler_t)(int);

#define SIG_DFL ((sighandler_t)0)
#define SIG_IGN ((sighandler_t)1)
#define SIG_ERR ((sighandler_t)-1)

#define SIGABRT 6
#define SIGFPE 8
#define SIGILL 4
#define SIGINT 2
#define SIGSEGV 11
#define SIGTERM 15

void (*signal(int sig, void (*handler)(int)))(int);
int raise(int sig);

#endif
"#;

static LOCALE_H: &str = r#"
#ifndef _LOCALE_H
#define _LOCALE_H

#include <stddef.h>

#define LC_ALL 0
#define LC_COLLATE 1
#define LC_CTYPE 2
#define LC_MONETARY 3
#define LC_NUMERIC 4
#define LC_TIME 5

struct lconv {
    char *decimal_point;
    char *thousands_sep;
    char *grouping;
    char *mon_decimal_point;
    char *mon_thousands_sep;
    char *mon_grouping;
    char *positive_sign;
    char *negative_sign;
    char *currency_symbol;
    char *int_curr_symbol;
    char frac_digits;
    char p_cs_precedes;
    char n_cs_precedes;
    char p_sep_by_space;
    char n_sep_by_space;
    char p_sign_posn;
    char n_sign_posn;
    char int_frac_digits;
    char int_p_cs_precedes;
    char int_n_cs_precedes;
    char int_p_sep_by_space;
    char int_n_sep_by_space;
    char int_p_sign_posn;
    char int_n_sign_posn;
};

char *setlocale(int category, const char *locale);
struct lconv *localeconv(void);

#endif
"#;

static SETJMP_H: &str = r#"
#ifndef _SETJMP_H
#define _SETJMP_H

typedef long jmp_buf[16];

int setjmp(jmp_buf env);
void longjmp(jmp_buf env, int val);

#endif
"#;

static MATH_H: &str = r#"
#ifndef _MATH_H
#define _MATH_H

#define HUGE_VAL (__builtin_huge_val())
#define HUGE_VALF (__builtin_huge_valf())
#define HUGE_VALL (__builtin_huge_vall())
#define INFINITY (__builtin_inff())
#define NAN (__builtin_nanf(""))

#define FP_INFINITE 1
#define FP_NAN 2
#define FP_NORMAL 3
#define FP_SUBNORMAL 4
#define FP_ZERO 0

double acos(double x);
double asin(double x);
double atan(double x);
double atan2(double y, double x);
double cos(double x);
double sin(double x);
double tan(double x);
double acosh(double x);
double asinh(double x);
double atanh(double x);
double cosh(double x);
double sinh(double x);
double tanh(double x);
double exp(double x);
double exp2(double x);
double expm1(double x);
double frexp(double x, int *exp);
double ldexp(double x, int exp);
double log(double x);
double log10(double x);
double log1p(double x);
double log2(double x);
double logb(double x);
double modf(double x, double *iptr);
double scalbn(double x, int n);
double scalbln(double x, long n);
double cbrt(double x);
double fabs(double x);
double hypot(double x, double y);
double pow(double x, double y);
double sqrt(double x);
double erf(double x);
double erfc(double x);
double tgamma(double x);
double lgamma(double x);
double ceil(double x);
double floor(double x);
double nearbyint(double x);
double rint(double x);
long lrint(double x);
long long llrint(double x);
double round(double x);
long lround(double x);
long long llround(double x);
double trunc(double x);
double fmod(double x, double y);
double remainder(double x, double y);
double remquo(double x, double y, int *quo);
double copysign(double x, double y);
double nan(const char *tagp);
double nextafter(double x, double y);
double nexttoward(double x, long double y);
double fdim(double x, double y);
double fmax(double x, double y);
double fmin(double x, double y);
double fma(double x, double y, double z);

#endif
"#;

static INTTYPES_H: &str = r#"
#ifndef _INTTYPES_H
#define _INTTYPES_H

#include <stdint.h>

#define PRId8 "d"
#define PRId16 "d"
#define PRId32 "d"
#define PRId64 "lld"
#define PRIi8 "i"
#define PRIi16 "i"
#define PRIi32 "i"
#define PRIi64 "lli"
#define PRIu8 "u"
#define PRIu16 "u"
#define PRIu32 "u"
#define PRIu64 "llu"
#define PRIx8 "x"
#define PRIx16 "x"
#define PRIx32 "x"
#define PRIx64 "llx"
#define PRIX8 "X"
#define PRIX16 "X"
#define PRIX32 "X"
#define PRIX64 "llX"
#define SCNd8 "hhd"
#define SCNd16 "hd"
#define SCNd32 "d"
#define SCNd64 "lld"

intmax_t imaxabs(intmax_t j);
imaxdiv_t imaxdiv(intmax_t numer, intmax_t denom);
intmax_t strtoimax(const char *nptr, char **endptr, int base);
uintmax_t strtoumax(const char *nptr, char **endptr, int base);

#endif
"#;

static UNISTD_H: &str = r#"
#ifndef _UNISTD_H
#define _UNISTD_H

#include <stddef.h>
#include <sys/types.h>

#define STDIN_FILENO 0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

#define F_OK 0
#define X_OK 1
#define W_OK 2
#define R_OK 4

int close(int fd);
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);
off_t lseek(int fd, off_t offset, int whence);
int pipe(int pipefd[2]);
int dup(int oldfd);
int dup2(int oldfd, int newfd);
int chdir(const char *path);
char *getcwd(char *buf, size_t size);
unsigned int sleep(unsigned int seconds);
int unlink(const char *pathname);
int access(const char *pathname, int mode);
pid_t getpid(void);
pid_t getppid(void);

#endif
"#;

static SYS_TYPES_H: &str = r#"
#ifndef _SYS_TYPES_H
#define _SYS_TYPES_H

#include <stddef.h>

typedef long off_t;
typedef long pid_t;
typedef unsigned long uid_t;
typedef unsigned long gid_t;
typedef long dev_t;
typedef unsigned long ino_t;
typedef unsigned short nlink_t;
typedef long time_t;
typedef long blksize_t;
typedef long long blkcnt_t;
typedef unsigned long mode_t;
typedef long ssize_t;

#endif
"#;

static SYS_STAT_H: &str = r#"
#ifndef _SYS_STAT_H
#define _SYS_STAT_H

#include <sys/types.h>
#include <time.h>

#define S_IFMT 0170000
#define S_IFSOCK 0140000
#define S_IFLNK 0120000
#define S_IFREG 0100000
#define S_IFBLK 0060000
#define S_IFDIR 0040000
#define S_IFCHR 0020000
#define S_IFIFO 0010000
#define S_ISUID 0004000
#define S_ISGID 0002000
#define S_ISVTX 0001000

struct stat {
    dev_t st_dev;
    ino_t st_ino;
    nlink_t st_nlink;
    mode_t st_mode;
    uid_t st_uid;
    gid_t st_gid;
    int __pad0;
    dev_t st_rdev;
    off_t st_size;
    blksize_t st_blksize;
    blkcnt_t st_blocks;
    time_t st_atime;
    time_t st_mtime;
    time_t st_ctime;
};

int stat(const char *path, struct stat *buf);
int fstat(int fd, struct stat *buf);
int lstat(const char *path, struct stat *buf);
mode_t umask(mode_t mask);
int mkdir(const char *path, mode_t mode);
int mknod(const char *path, mode_t mode, dev_t dev);

#endif
"#;

static FCNTL_H: &str = r#"
#ifndef _FCNTL_H
#define _FCNTL_H

#include <sys/types.h>
#include <sys/stat.h>

#define O_RDONLY 0
#define O_WRONLY 1
#define O_RDWR 2
#define O_CREAT 0100
#define O_EXCL 0200
#define O_NOCTTY 0400
#define O_TRUNC 01000
#define O_APPEND 02000
#define O_NONBLOCK 04000
#define O_DSYNC 010000
#define O_DIRECTORY 0200000
#define O_NOFOLLOW 0400000
#define O_CLOEXEC 02000000
#define O_SYNC 04010000

int open(const char *pathname, int flags, ...);
int creat(const char *pathname, mode_t mode);
int fcntl(int fd, int cmd, ...);

#endif
"#;

static PTHREAD_H: &str = r#"
#ifndef _PTHREAD_H
#define _PTHREAD_H

#include <stddef.h>
#include <sys/types.h>
#include <time.h>

typedef unsigned long pthread_t;
typedef unsigned long pthread_key_t;
typedef unsigned long pthread_mutex_t;
typedef unsigned long pthread_cond_t;
typedef unsigned long pthread_rwlock_t;
typedef unsigned long pthread_once_t;
typedef unsigned long pthread_attr_t;
typedef unsigned long pthread_mutexattr_t;
typedef unsigned long pthread_condattr_t;
typedef unsigned long pthread_rwlockattr_t;
typedef struct { unsigned long __data; } pthread_spinlock_t;

#define PTHREAD_ONCE_INIT 0
#define PTHREAD_CREATE_JOINABLE 0
#define PTHREAD_CREATE_DETACHED 1
#define PTHREAD_MUTEX_NORMAL 0
#define PTHREAD_MUTEX_ERRORCHECK 1
#define PTHREAD_MUTEX_RECURSIVE 2
#define PTHREAD_MUTEX_DEFAULT 0

int pthread_create(pthread_t *thread, const pthread_attr_t *attr, void *(*start_routine)(void *), void *arg);
int pthread_join(pthread_t thread, void **retval);
int pthread_detach(pthread_t thread);
pthread_t pthread_self(void);
int pthread_equal(pthread_t t1, pthread_t t2);
void pthread_exit(void *retval);
int pthread_mutex_init(pthread_mutex_t *mutex, const pthread_mutexattr_t *attr);
int pthread_mutex_destroy(pthread_mutex_t *mutex);
int pthread_mutex_lock(pthread_mutex_t *mutex);
int pthread_mutex_unlock(pthread_mutex_t *mutex);
int pthread_mutex_trylock(pthread_mutex_t *mutex);
int pthread_cond_init(pthread_cond_t *cond, const pthread_condattr_t *attr);
int pthread_cond_destroy(pthread_cond_t *cond);
int pthread_cond_wait(pthread_cond_t *cond, pthread_mutex_t *mutex);
int pthread_cond_signal(pthread_cond_t *cond);
int pthread_cond_broadcast(pthread_cond_t *cond);
int pthread_once(pthread_once_t *once_control, void (*init_routine)(void));
int pthread_key_create(pthread_key_t *key, void (*destructor)(void *));
void *pthread_getspecific(pthread_key_t key);
int pthread_setspecific(pthread_key_t key, const void *value);

#endif
"#;

static DLFCN_H: &str = r#"
#ifndef _DLFCN_H
#define _DLFCN_H

#define RTLD_LAZY 1
#define RTLD_NOW 2
#define RTLD_GLOBAL 256
#define RTLD_LOCAL 0
#define RTLD_DEFAULT ((void*)0)
#define RTLD_NEXT ((void*)-1)

void *dlopen(const char *filename, int flags);
char *dlerror(void);
void *dlsym(void *handle, const char *symbol);
int dlclose(void *handle);

#endif
"#;

static STDNORETURN_H: &str = r#"
#ifndef _STDNORETURN_H
#define _STDNORETURN_H

#define noreturn _Noreturn

#endif
"#;

static STDALIGN_H: &str = r#"
#ifndef _STDALIGN_H
#define _STDALIGN_H

#define alignas _Alignas
#define alignof _Alignof
#define __alignas_is_defined 1
#define __alignof_is_defined 1

#endif
"#;

static UCHAR_H: &str = r#"
#ifndef _UCHAR_H
#define _UCHAR_H

#include <stddef.h>

typedef unsigned int char16_t;
typedef unsigned int char32_t;

size_t c16rtomb(char *s, char16_t c16, mbstate_t *ps);
size_t c32rtomb(char *s, char32_t c32, mbstate_t *ps);
size_t mbrtoc16(char16_t *pc16, const char *s, size_t n, mbstate_t *ps);
size_t mbrtoc32(char32_t *pc32, const char *s, size_t n, mbstate_t *ps);

#endif
"#;

static COMPLEX_H: &str = r#"
#ifndef _COMPLEX_H
#define _COMPLEX_H

#define complex _Complex
#define _Complex_I (1.0fi)
#define I _Complex_I

#endif
"#;

static TGMATH_H: &str = r#"
#ifndef _TGMATH_H
#define _TGMATH_H

#include <math.h>
#include <complex.h>

#endif
"#;

static ISO646_H: &str = r#"
#ifndef _ISO646_H
#define _ISO646_H

#define and &&
#define or ||
#define not !
#define and_eq &=
#define or_eq |=
#define not_eq !=
#define bitand &
#define bitor |
#define xor ^
#define xor_eq ^=

#endif
"#;

static STDATOMIC_H: &str = r#"
#ifndef _STDATOMIC_H
#define _STDATOMIC_H

#define ATOMIC_BOOL_LOCK_FREE 2
#define ATOMIC_CHAR_LOCK_FREE 2
#define ATOMIC_CHAR16_T_LOCK_FREE 2
#define ATOMIC_CHAR32_T_LOCK_FREE 2
#define ATOMIC_WCHAR_T_LOCK_FREE 2
#define ATOMIC_SHORT_LOCK_FREE 2
#define ATOMIC_INT_LOCK_FREE 2
#define ATOMIC_LONG_LOCK_FREE 2
#define ATOMIC_LLONG_LOCK_FREE 2
#define ATOMIC_POINTER_LOCK_FREE 2

typedef _Atomic _Bool atomic_bool;
typedef _Atomic char atomic_char;
typedef _Atomic signed char atomic_schar;
typedef _Atomic unsigned char atomic_uchar;
typedef _Atomic short atomic_short;
typedef _Atomic unsigned short atomic_ushort;
typedef _Atomic int atomic_int;
typedef _Atomic unsigned int atomic_uint;
typedef _Atomic long atomic_long;
typedef _Atomic unsigned long atomic_ulong;
typedef _Atomic long long atomic_llong;
typedef _Atomic unsigned long long atomic_ullong;
typedef _Atomic char16_t atomic_char16_t;
typedef _Atomic char32_t atomic_char32_t;
typedef _Atomic wchar_t atomic_wchar_t;
typedef _Atomic int_least8_t atomic_int_least8_t;
typedef _Atomic uint_least8_t atomic_uint_least8_t;
typedef _Atomic int_least16_t atomic_int_least16_t;
typedef _Atomic uint_least16_t atomic_uint_least16_t;
typedef _Atomic int_least32_t atomic_int_least32_t;
typedef _Atomic uint_least32_t atomic_uint_least32_t;
typedef _Atomic int_least64_t atomic_int_least64_t;
typedef _Atomic uint_least64_t atomic_uint_least64_t;
typedef _Atomic int_fast8_t atomic_int_fast8_t;
typedef _Atomic uint_fast8_t atomic_uint_fast8_t;
typedef _Atomic int_fast16_t atomic_int_fast16_t;
typedef _Atomic uint_fast16_t atomic_uint_fast16_t;
typedef _Atomic int_fast32_t atomic_int_fast32_t;
typedef _Atomic uint_fast32_t atomic_uint_fast32_t;
typedef _Atomic int_fast64_t atomic_int_fast64_t;
typedef _Atomic uint_fast64_t atomic_uint_fast64_t;
typedef _Atomic intptr_t atomic_intptr_t;
typedef _Atomic uintptr_t atomic_uintptr_t;
typedef _Atomic size_t atomic_size_t;
typedef _Atomic ptrdiff_t atomic_ptrdiff_t;
typedef _Atomic intmax_t atomic_intmax_t;
typedef _Atomic uintmax_t atomic_uintmax_t;

void atomic_init(volatile A *obj, C desired);

#define ATOMIC_VAR_INIT(value) (value)
#define atomic_is_lock_free(obj) (__c11_atomic_is_lock_free(sizeof(*(obj))))

#endif
"#;

static THREADS_H: &str = r#"
#ifndef _THREADS_H
#define _THREADS_H

#include <stddef.h>
#include <time.h>

typedef int thrd_t;
typedef unsigned long thrd_start_t;
typedef void *tx_state;
typedef unsigned long mtx_t;
typedef unsigned long cnd_t;
typedef unsigned long tss_t;

#define thread_local _Thread_local
#define TSS_DTOR_ITERATIONS 4
#define ONCE_FLAG_INIT 0
#define CALL_ONCE __ONCE_FLAG_INIT

#define thrd_success 0
#define thrd_timedout 1
#define thrd_busy 2
#define thrd_error 3
#define thrd_nomem 4

int thrd_create(thrd_t *thr, thrd_start_t func, void *arg);
int thrd_equal(thrd_t thr0, thrd_t thr1);
thrd_t thrd_current(void);
int thrd_detach(thrd_t thr);
int thrd_join(thrd_t thr, int *res);
void thrd_exit(int res);
int thrd_sleep(const struct timespec *duration, struct timespec *remaining);
void thrd_yield(void);
int mtx_init(mtx_t *mtx, int type);
void mtx_destroy(mtx_t *mtx);
int mtx_lock(mtx_t *mtx);
int mtx_timedlock(mtx_t *mtx, const struct timespec *until);
int mtx_trylock(mtx_t *mtx);
int mtx_unlock(mtx_t *mtx);
int cnd_init(cnd_t *cond);
void cnd_destroy(cnd_t *cond);
int cnd_signal(cnd_t *cond);
int cnd_broadcast(cnd_t *cond);
int cnd_wait(cnd_t *cond, mtx_t *mtx);
int cnd_timedwait(cnd_t *cond, mtx_t *mtx, const struct timespec *until);
int tss_create(tss_t *key, tss_dtor_t dtor);
void tss_delete(tss_t key);
void *tss_get(tss_t key);
int tss_set(tss_t key, void *val);
void call_once(once_flag *flag, void (*func)(void));

#endif
"#;
