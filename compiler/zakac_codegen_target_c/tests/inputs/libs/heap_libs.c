/* Links two libs of heaplib.zaka, each built with --max-heap 800000: each
   counts the heap its calls hold under its own cap, so both hold a list of
   65536 cells, and a lib that grows past its cap stops on its own. */
#include <stdint.h>
#include <stdio.h>

#include "heapa.h"
#include "heapb.h"

int main(void)
{
    uint64_t count = 0;
    int status = fill_a(65536, &count);
    printf("a %d %u\n", status, (unsigned)count);
    status = fill_b(65536, &count);
    printf("b %d %u\n", status, (unsigned)count);
    status = fill_a(65536, &count);
    const zk_trap *trap = zk_last_trap_from_heapa();
    printf("a again %d %s\n", status == trap->code + 1, trap->kind);
    return 0;
}
