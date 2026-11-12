/* The print of each float kind, worked out on the bits as the VM writes it.
 * zaka_rt.h includes it at its end. */

#ifndef ZAKA_RT_FLOAT_H
#define ZAKA_RT_FLOAT_H

#include <stdint.h>

/** Write number text with Burmese digits; zaka_rt.c holds it. */
void zk_put_burmese(const char *s);
/** Write a float from its bit pattern, as the VM writes it. */
void zk_put_fbits(int bits, uint64_t raw);

#endif
