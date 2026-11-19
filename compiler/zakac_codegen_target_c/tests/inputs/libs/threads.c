/* Calls the lib of mathlib.zaka from several threads at once: the lib lets
   one call in at a time, so each count comes out right. Windows runs the
   threads through its own API, each other OS through pthreads. */
#include <stdint.h>
#include <stdio.h>

#include "mathlib.h"

#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#endif

#define THREADS 8
#define CALLS 2000

/* The sum each thread counts. */
static uint64_t sums[THREADS];

/**
 * Count the digits of the numbers from 0, through the lib, into the sum of
 * a thread; a call that fails leaves the sum at 0.
 *
 * @param at The thread.
 */
static void count_digits(int at)
{
    uint64_t sum = 0;
    for (int64_t n = 0; n < CALLS; n++)
    {
        uint64_t count = 0;
        if (digit_count(n, &count) != ZK_OK)
            return;
        sum += count;
    }
    sums[at] = sum;
}

#ifdef _WIN32
/**
 * The work of a thread.
 *
 * @param arg The thread, as an address.
 * @return 0.
 */
static DWORD WINAPI run(LPVOID arg)
{
    count_digits((int)(intptr_t)arg);
    return 0;
}

int main(void)
{
    HANDLE threads[THREADS];
    for (int i = 0; i < THREADS; i++)
        threads[i] = CreateThread(NULL, 0, run, (LPVOID)(intptr_t)i, 0, NULL);
    WaitForMultipleObjects(THREADS, threads, TRUE, INFINITE);
    uint64_t total = 0;
    for (int i = 0; i < THREADS; i++)
        total += sums[i];
    printf("%llu\n", (unsigned long long)total);
    return 0;
}
#else
/**
 * The work of a thread.
 *
 * @param arg The thread, as an address.
 * @return NULL.
 */
static void *run(void *arg)
{
    count_digits((int)(intptr_t)arg);
    return NULL;
}

int main(void)
{
    pthread_t threads[THREADS];
    for (int i = 0; i < THREADS; i++)
        pthread_create(&threads[i], NULL, run, (void *)(intptr_t)i);
    for (int i = 0; i < THREADS; i++)
        pthread_join(threads[i], NULL);
    uint64_t total = 0;
    for (int i = 0; i < THREADS; i++)
        total += sums[i];
    printf("%llu\n", (unsigned long long)total);
    return 0;
}
#endif
