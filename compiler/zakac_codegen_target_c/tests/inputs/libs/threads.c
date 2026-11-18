/* Calls the lib of mathlib.zaka from several threads at once: the lib lets
   one call in at a time, so each count comes out right. */
#include <pthread.h>
#include <stdio.h>

#include "mathlib.h"

#define THREADS 8
#define CALLS 2000

/**
 * Count the digits of the numbers from 0, through the lib.
 *
 * @param arg Unused.
 * @return The sum of the counts, as an address.
 */
static void *count_digits(void *arg)
{
    (void)arg;
    uintptr_t sum = 0;
    for (int64_t n = 0; n < CALLS; n++)
    {
        uint64_t count = 0;
        if (digit_count(n, &count) != ZK_OK)
            return NULL;
        sum += (uintptr_t)count;
    }
    return (void *)sum;
}

int main(void)
{
    pthread_t threads[THREADS];
    for (int i = 0; i < THREADS; i++)
        pthread_create(&threads[i], NULL, count_digits, NULL);
    uintptr_t total = 0;
    for (int i = 0; i < THREADS; i++)
    {
        void *sum = NULL;
        pthread_join(threads[i], &sum);
        total += (uintptr_t)sum;
    }
    printf("%lu\n", (unsigned long)total);
    return 0;
}
