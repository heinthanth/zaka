/* Links two Zaka libs into one program: each keeps its own names and shares
   the runtime with the other, and a trap stops the lib it came from alone. */
#include <stdint.h>
#include <stdio.h>

#include "mathlib.h"
#include "twicelib.h"

int main(void)
{
    int32_t sum = 0;
    int32_t doubled = 0;
    int64_t quotient = 0;
    /* Each call runs before the print that reads its result: C leaves the
       order of the arguments of a call open. */
    int status = add_numbers(2, 40, &sum);
    printf("add %d %d\n", status, sum);
    status = twice(21, &doubled);
    printf("twice %d %d\n", status, doubled);
    printf("divide %d\n", divide(7, 0, &quotient));
    printf("traps %s %s\n", zk_last_trap_from_mathlib() != NULL ? "math" : "none",
           zk_last_trap_from_twicelib() != NULL ? "twice" : "none");
    printf("latest %s\n", zk_last_trap()->lib);
    status = add_numbers(1, 1, &sum);
    printf("after %d", status);
    status = twice(5, &doubled);
    printf(" %d %d\n", status, doubled);
    zk_reset_mathlib();
    status = add_numbers(1, 2, &sum);
    printf("reset %d %d\n", status, sum);
    return 0;
}
