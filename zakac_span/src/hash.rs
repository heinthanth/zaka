use sha2::Digest;
use sha2::Sha256;

/// Algorithm used to hash source file contents.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum SourceFileHashAlgorithm
{
    /// SHA-256 over the original UTF-8 content.
    #[default]
    Sha256,
}

/// Hash of a source file's original content.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
        SourceFileHash { algorithm, hash }
    }

    /// Hash `src` with the given algorithm.
    ///
    /// # Arguments
    ///
    /// * `algo` - Hash algorithm to use.
    /// * `src` - Original source text (pre-normalize).
    ///
    /// # Returns
    ///
    /// The computed hash.
    pub fn compute(algo: SourceFileHashAlgorithm, src: &str) -> Self
    {
        match algo
        {
            SourceFileHashAlgorithm::Sha256 => Self::new(algo, Self::calc_sha256(src)),
        }
    }

    /// Make a SHA-256 hash of `src`.
    ///
    /// # Arguments
    ///
    /// * `src` - The source text to hash.
    ///
    /// # Returns
    ///
    /// The SHA-256 hash of `src`.
    fn calc_sha256(src: &str) -> [u8; 32]
    {
        let digest = Sha256::digest(src.as_bytes());
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest);
        hash
    }
}
