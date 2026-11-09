/* A small C lib the FFI tests call from Zaka, in the VM and as an executable: a
   struct read through `...`, text as a result, and a Zaka function called
   back from C. */
#include <stdarg.h>
#include <stdint.h>
#include <stddef.h>

/** A point, as Zaka holds a struct of two 32-bit integers. */
typedef struct
{
    int32_t a;
    int32_t b;
} zk_point;

/**
 * The sum of a * 10 + b over each point given after the count.
 *
 * @param count The count of points that follow.
 * @return The sum.
 */
int64_t point_sum(int32_t count, ...)
{
    va_list ap;
    va_start(ap, count);
    int64_t sum = 0;
    for (int32_t i = 0; i < count; i++)
    {
        zk_point p = va_arg(ap, zk_point);
        sum += (int64_t)p.a * 10 + p.b;
    }
    va_end(ap);
    return sum;
}

/**
 * A name, or NULL.
 *
 * @param good Whether to give the name.
 * @return "zaka", or NULL when `good` is 0.
 */
const char *name_or_null(int32_t good)
{
    return good ? "zaka" : NULL;
}

/**
 * Call a function twice, on a value and then on what it gave back.
 *
 * @param f The function.
 * @param v The value.
 * @return f(f(v)).
 */
int64_t call_twice(int64_t (*f)(int64_t), int64_t v)
{
    return f(f(v));
}
