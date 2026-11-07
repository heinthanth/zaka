/* One-line sugars for control flow in the C runtime similar to zakac_rust_sugars */

#ifndef ZAKA_C_SUGARS_H
#define ZAKA_C_SUGARS_H

#include <stddef.h>

/**
 * Return when cond holds, with the value when one is given:
 *   RET_IF(n == 0);
 *   RET_IF(v < 0, -1);
 * C11 wants an argument for each `...`, so RET_IF picks RET_IF_VALUE or
 * RET_IF_VOID from its count of arguments.
 *
 * @param cond The condition.
 * @param value The value to return, left out in a function with no result.
 */
/* clang-format off */
#define RET_IF(...) RET_IF_PICK(__VA_ARGS__, RET_IF_VALUE, RET_IF_VOID, )(__VA_ARGS__)
#define RET_IF_PICK(a, b, name, ...) name
#define RET_IF_VALUE(cond, value) do { if (cond) return value; } while (0)
#define RET_IF_VOID(cond) do { if (cond) return; } while (0)
/* clang-format on */

/**
 * Run a statement, then return, when cond holds; for a function with no
 * result, where RET_IF cannot hold a call:
 *   DONE_IF(text, fputs(text, stdout));
 *
 * @param cond The condition.
 * @param stmt The statement to run before the return.
 */
/* clang-format off */
#define DONE_IF(cond, stmt) do { if (cond) { stmt; return; } } while (0)
/* clang-format on */

/**
 * Run a statement when cond holds:
 *   DO_IF(v < 0, fputc('-', stdout));
 *
 * @param cond The condition.
 * @param stmt The statement.
 */
/* clang-format off */
#define DO_IF(cond, stmt) do { if (cond) stmt; } while (0)
/* clang-format on */

/**
 * Go on with the next round of the loop around it when cond holds. A
 * do-while wrapper would catch the continue, so this is a bare if.
 *
 * @param cond The condition.
 */
/* clang-format off */
#define NEXT_IF(cond) if (cond) continue
/* clang-format on */

/**
 * Leave the loop around it when cond holds; a bare if, as NEXT_IF.
 *
 * @param cond The condition.
 */
/* clang-format off */
#define LEAVE_IF(cond) if (cond) break
/* clang-format on */

/**
 * Run the statement or block after it once for each i from 0 up to n, n
 * left out:
 *   DO_FOR(i, n) out[i] = '0';
 *
 * @param i The name of the counter, a `size_t`.
 * @param n The count of rounds.
 */
/* clang-format off */
#define DO_FOR(i, n) for (size_t i = 0; i < (size_t)(n); i++)
/* clang-format on */

#endif
