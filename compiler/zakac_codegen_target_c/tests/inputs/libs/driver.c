/* Calls the lib of mathlib.zaka: a sum, text in, a struct, a trap, then a call
   after it. */
#include <stdio.h>

#include "mathlib.h"

int main(void)
{
    /* The lib writes its own output, so each line goes out in order. */
    setvbuf(stdout, NULL, _IONBF, 0);
    int32_t sum = 0;
    int32_t status = add_numbers(2, 40, &sum);
    printf("add %d %d\n", status, sum);
    status = add_numbers(1, 2, NULL);
    printf("no result %d\n", status);
    uint64_t count = 0;
    status = greet("Zaka", &count);
    printf("greet %d %llu\n", status, (unsigned long long)count);
    point corner = {3, 4};
    int32_t length = 0;
    status = manhattan(corner, &length);
    printf("manhattan %d %d\n", status, length);
    printf("trap before %s\n", zk_last_trap() == NULL ? "none" : "some");
    int64_t quotient = 0;
    status = divide(7, 0, &quotient);
    printf("divide %d\n", status);
    printf("trap %s\n", zk_last_trap());
    status = add_numbers(1, 1, &sum);
    printf("after %d\n", status);
    return 0;
}
