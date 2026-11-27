/* Calls spinlib.zaka and flaglib.zaka, whose exports each write a global of
   their own lib. On a thread of its own, spin_until_set turns in Zaka code
   under the lock of spinlib until set_flag of flaglib sets the flag. Each lib
   has a lock of its own, so set_flag runs while spin_until_set holds the lock
   of spinlib; one lock for both libs would leave the flag unset. */
#define _POSIX_C_SOURCE 199309L
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <time.h>
#include "flaglib.h"
#include "spinlib.h"

static int64_t flag;

static void *spin(void *out)
{
    spin_until_set(&flag, (int64_t *)out);
    return NULL;
}

int main(void)
{
    int64_t seen = 0;
    int64_t sets = 0;
    pthread_t t;
    pthread_create(&t, NULL, spin, &seen);
    /* The spin starts first and holds the lock of spinlib. */
    struct timespec pause = {0, 100 * 1000000};
    nanosleep(&pause, NULL);
    int status = set_flag(&flag, &sets);
    pthread_join(t, NULL);
    printf("seen %lld set %d %lld\n", (long long)seen, status, (long long)sets);
    return 0;
}
