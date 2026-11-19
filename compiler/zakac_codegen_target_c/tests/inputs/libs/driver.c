/* Calls the lib of mathlib.zaka: a sum, text in, a struct, a trap, a call
   after it, then a call after a reset. */
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
    point flipped = {0, 0};
    status = mirror(corner, &flipped);
    printf("mirror %d %d %d\n", status, flipped.x, flipped.y);
    char *said = NULL;
    status = echo("hi", &said);
    printf("echo %d %s\n", status, said);
    zk_free(said);
    printf("echo with no out %d\n", echo("x", NULL));
    zk_list_int32_t squared = {NULL, 0};
    status = squares(7, &squared);
    printf("squares %d %d %d %d\n", status, (int)squared.count, squared.ptr[0], squared.ptr[1]);
    zk_free(squared.ptr);
    char ann[] = "Ann";
    person given = {ann, 30};
    person grown = {NULL, 0};
    status = older(given, &grown);
    printf("older %d %s %d\n", status, grown.name, grown.age);
    zk_free(grown.name);
    printf("trap before %s\n", zk_last_trap() == NULL ? "none" : "some");
    int64_t quotient = 0;
    status = divide(7, 0, &quotient);
    printf("divide %d\n", status);
    const zk_trap *trap = zk_last_trap_from_mathlib();
    printf("trap %s\n", trap->message);
    printf("trap code %d %d %s %s\n", trap->code == ZK_TRAP_ZERO_DIVISOR,
           status == trap->code + 1, trap->kind, trap->lib);
    printf("trap at %s %s:%u:%u, %u calls, the latest %d\n", trap->where.function,
           trap->where.file, trap->where.line, trap->where.column,
           (unsigned)trap->frame_count, zk_last_trap() == trap);
    status = add_numbers(1, 1, &sum);
    printf("after %d\n", status);
    zk_reset_mathlib();
    printf("trap after reset %s\n", zk_last_trap_from_mathlib() == NULL ? "none" : "some");
    status = add_numbers(1, 2, &sum);
    printf("reset %d %d\n", status, sum);
    return 0;
}
