/* A small C lib the FFI tests call from Zaka, in the VM and as an executable: a
   struct read through `...`, text as a result, and Zaka functions called
   back from C with numbers, text, structs and handles. */
#include <stdarg.h>
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>

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
 * Swap the two fields of a point in place.
 *
 * @param p The point.
 */
void point_swap(zk_point *p)
{
    int32_t a = p->a;
    p->a = p->b;
    p->b = a;
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

/** A counter C keeps behind a handle. */
typedef struct zk_counter
{
    int32_t count;
    int32_t step;
    const char *label;
    int32_t marks[2];
    struct zk_counter *self_ref;
} zk_counter;

/**
 * A new counter, or NULL for a step of 0.
 *
 * @param step What each step adds.
 * @return The counter, or NULL.
 */
zk_counter *counter_open(int32_t step)
{
    if (step == 0)
    {
        return NULL;
    }
    zk_counter *c = malloc(sizeof *c);
    c->count = 0;
    c->step = step;
    c->label = step >= 5 ? "big" : NULL;
    c->marks[0] = step;
    c->marks[1] = step * 2;
    c->self_ref = step >= 5 ? c : NULL;
    return c;
}

/**
 * Add one step to a counter.
 *
 * @param c The counter.
 * @return The count after the step.
 */
int32_t counter_next(zk_counter *c)
{
    c->count += c->step;
    return c->count;
}

/**
 * Free a counter.
 *
 * @param c The counter.
 */
void counter_close(zk_counter *c) { free(c); }

/**
 * The sum of the steps of each counter given after the count.
 *
 * @param count The count of counters that follow.
 * @return The sum.
 */
int32_t counter_steps(int32_t count, ...)
{
    va_list ap;
    va_start(ap, count);
    int32_t sum = 0;
    for (int32_t i = 0; i < count; i++)
    {
        zk_counter *c = va_arg(ap, zk_counter *);
        sum += c->step;
    }
    va_end(ap);
    return sum;
}

/**
 * Call a function on a text, or on NULL.
 *
 * @param f The function, which takes an optional of text.
 * @param null Whether to pass NULL.
 * @return What f gave back.
 */
int32_t text_call(int32_t (*f)(const char *), int32_t null)
{
    return f(null ? NULL : "hello");
}

/**
 * Call a function on a text, or on NULL, which stops the run of a function
 * that takes plain text.
 *
 * @param f The function, which takes text.
 * @param null Whether to pass NULL.
 * @return What f gave back.
 */
int32_t plain_text_call(int32_t (*f)(const char *), int32_t null)
{
    return f(null ? NULL : "plain");
}

/**
 * Call a function on a point, passed as a value.
 *
 * @param f The function.
 * @return What f gave back.
 */
int32_t point_call(int32_t (*f)(zk_point))
{
    zk_point p = {3, 4};
    return f(p);
}

/** A line: a point, two marks and a weight, which C passes as a value. */
typedef struct
{
    zk_point from;
    int32_t marks[2];
    double weight;
} zk_line;

/**
 * Call a function on a line, passed as a value.
 *
 * @param f The function.
 * @return What f gave back.
 */
int32_t line_call(int32_t (*f)(zk_line))
{
    zk_line l = {{1, 2}, {3, 4}, 0.5};
    return f(l);
}

/**
 * Call a function on a counter, or on NULL.
 *
 * @param f The function, which takes a handle.
 * @param c The counter.
 * @param null Whether to pass NULL.
 * @return What f gave back.
 */
int32_t counter_call(int32_t (*f)(zk_counter *), zk_counter *c, int32_t null)
{
    return f(null ? NULL : c);
}

/**
 * Call a function on a counter, or on NULL.
 *
 * @param f The function, which takes an optional of a handle.
 * @param c The counter.
 * @param null Whether to pass NULL.
 * @return What f gave back.
 */
int32_t opt_counter_call(int32_t (*f)(zk_counter *), zk_counter *c, int32_t null)
{
    return f(null ? NULL : c);
}
