#ifndef _STDDEF_H
#define _STDDEF_H

typedef __typeof__(sizeof(0)) size_t;
typedef __typeof__((char *)0 - (char *)0) ptrdiff_t;
typedef __typeof__(0) wchar_t;

#ifndef NULL
#define NULL ((void *)0)
#endif

#define offsetof(type, member) __builtin_offsetof(type, member)

#endif
