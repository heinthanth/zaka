use bon::Builder;
use sha2::Digest;
use sha2::Sha256;

/// Algorithm used to hash source file contents.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum SourceFileHashAlgorithm
{
    /// SHA-256 over the original file u8s.
    #[default]
    Sha256,
}

/// Hash of a source file's original content.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Builder)]
pub struct SourceFileHash
{
    /// Algorithm used.
    pub algorithm: SourceFileHashAlgorithm,
    /// Hash value.
    pub hash: [u8; 32],
}

impl SourceFileHash
{
    /// Create a new source file hash.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The algorithm to use.
    /// * `hash` - The hash to create a source file hash from.
    ///
    /// # Returns
    ///
    /// The new source file hash.
    pub fn new(algorithm: SourceFileHashAlgorithm, hash: [u8; 32]) -> Self
    {
        SourceFileHash::builder().algorithm(algorithm).hash(hash).build()
    }

    /// Make a SHA-256 hash of `src`.
    ///
    /// # Arguments
    ///
    /// * `src` - The u8s to hash.
    ///
    /// # Returns
    ///
    /// The SHA-256 hash of `src`.
    fn calc_sha256(src: &[u8]) -> [u8; 32]
    {
        let digest = Sha256::digest(src);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest);
        hash
    }

    /// Hash `src` with the given algorithm.
    ///
    /// # Arguments
    ///
    /// * `algo` - Hash algorithm to use.
    /// * `src` - Original file u8s (pre-normalize for text sources).
    ///
    /// # Returns
    ///
    /// The computed hash.
    pub fn compute(algo: SourceFileHashAlgorithm, src: &[u8]) -> Self
    {
        match algo
        {
            SourceFileHashAlgorithm::Sha256 => Self::new(algo, Self::calc_sha256(src)),
        }
    }
}
