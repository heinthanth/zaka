/* Calls freelib.zaka from eight threads at once. fresh_sum and divide reach
   no mutable global, so their calls run with no lock; count_call writes one,
   so its calls hold the lock of the lib. A trap in divide fails that call
   alone: the lib goes on. */
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include "freelib.h"

#define THREADS 8
#define ROUNDS 5000

static void *work(void *arg)
{
    intptr_t me = (intptr_t)arg;
    intptr_t wrong = 0;
    for (int i = 0; i < ROUNDS; i++)
    {
        int64_t n = 0;
        int status = fresh_sum(me * ROUNDS + i, &n);
        if (status != 0 || n != me * ROUNDS + i + 9) wrong++;
        if (i % 100 == 0 && count_call(&n) != 0) wrong++;
    }
    return (void *)wrong;
}

int main(void)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    pthread_t threads[THREADS];
    for (intptr_t t = 0; t < THREADS; t++)
        pthread_create(&threads[t], NULL, work, (void *)t);
    int64_t q = 0;
    int failed = divide(1, 0, &q);
    intptr_t wrong = 0;
    for (int t = 0; t < THREADS; t++)
    {
        void *got = NULL;
        pthread_join(threads[t], &got);
        wrong += (intptr_t)got;
    }
    printf("wrong %ld\n", (long)wrong);
    const zk_trap *trap = zk_last_trap_from_freelib();
    printf("fail %s %s\n", failed != 0 ? "trap" : "none",
           trap != NULL ? trap->kind : "none");
    int status = divide(6, 3, &q);
    printf("after %d %lld\n", status, (long long)q);
    status = count_call(&q);
    printf("count %d %lld\n", status, (long long)q);
    return 0;
}
