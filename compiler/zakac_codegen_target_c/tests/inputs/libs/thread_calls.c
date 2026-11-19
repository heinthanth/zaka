/* Calls threadlib.zaka, whose functions start a thread through pthread and
   wait for it in pthread_join: while the call waits in C the lib lets the
   thread in, which calls a function of the program. A trap on that thread
   stops the lib; the call that waits goes on. */
#include <stdint.h>
#include <stdio.h>
#include "threadlib.h"
int main(void)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    int64_t n = 0;
    int status = run_on_a_thread(1, &n);
    printf("thread %d %lld\n", status, (long long)n);
    status = run_on_a_thread(10, &n);
    printf("thread %d %lld\n", status, (long long)n);
    status = fail_on_a_thread(1, &n);
    printf("fail %d %lld\n", status, (long long)n);
    const zk_trap *trap = zk_last_trap_from_threadlib();
    printf("trap %s in %s\n", trap != NULL ? trap->kind : "none",
           trap != NULL && trap->where.function != NULL ? trap->where.function : "-");
    status = run_on_a_thread(1, &n);
    printf("after %d\n", status);
    zk_reset_threadlib();
    status = run_on_a_thread(1, &n);
    printf("reset %d %lld\n", status, (long long)n);
    return 0;
}
