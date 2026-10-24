// Shared with `build.rs`, which pulls this file in with `include!`; keep it
// free of crate imports.

/// Seed of the mask stream.
pub const SEED: u64 = 0x7a61_6b61_636f_6465;

/// XOR `raw` with an xorshift64 stream that starts at `seed`. Masking twice
/// gives the input back.
///
/// # Arguments
///
/// * `raw` - The u8s to mask, in place.
/// * `seed` - The seed of the stream.
pub fn xor_mask(raw: &mut [u8], seed: u64)
{
    let mut state = seed | 1;
    for chunk in raw.chunks_mut(8)
    {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        chunk
            .iter_mut()
            .zip(state.to_le_bytes())
            .for_each(|(cell, mask)| *cell ^= mask);
    }
}
