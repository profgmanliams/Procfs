use crate::{expect, FromBufRead, ProcError, ProcResult};

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::TryFrom,
    io::BufRead,
    iter::{once, Peekable},
    str::FromStr,
};

/// Represents the data from `/proc/crypto`.
///
/// Each block represents a cryptographic implementation that has been registered with the kernel.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct CryptoTable {
    pub crypto_blocks: HashMap<String, Vec<CryptoBlock>>,
}

/// Format of a crypto implementation represented in /proc/crypto.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct CryptoBlock {
    pub name: String,
    pub driver: String,
    pub module: String,
    pub priority: isize,
    pub ref_count: isize,
    pub self_test: SelfTest,
    pub internal: bool,
    pub fips_enabled: bool,
    pub crypto_type: Type,
}

impl FromBufRead for CryptoTable {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut lines = r.lines().peekable();
        let mut crypto_blocks: HashMap<String, Vec<CryptoBlock>> = HashMap::new();
        while let Some(line) = lines.next() {
            let line = line?;
            // Just skip empty lines
            if !line.is_empty() {
                let mut split = line.split(':');
                let name = expect!(split.next());
                if name.trim() == "name" {
                    let name = expect!(split.next()).trim().to_string();
                    let block = CryptoBlock::from_iter(&mut lines, name.as_str())?;
                    let blocks = crypto_blocks.entry(name).or_insert(Vec::new());
                    blocks.push(block);
                }
            }
        }

        Ok(CryptoTable { crypto_blocks })
    }
}

impl CryptoTable {
    pub fn get<T: AsRef<str>>(&self, target: T) -> Option<&Vec<CryptoBlock>> {
        self.crypto_blocks.get(target.as_ref())
    }
}

impl CryptoBlock {
    fn from_iter<T: Iterator<Item = Result<String, std::io::Error>>>(
        iter: &mut Peekable<T>,
        name: &str,
    ) -> ProcResult<Self> {
        let driver = parse_line(iter, "driver", name)?;
        let module = parse_line(iter, "module", name)?;
        let priority = from_str!(isize, &parse_line(iter, "priority", name)?);
        let ref_count = from_str!(isize, &parse_line(iter, "refcnt", name)?);
        let self_test = SelfTest::try_from(parse_line(iter, "selftest", name)?.as_str())?;
        let internal = parse_bool(iter, "internal", name)?;
        let fips_enabled = parse_fips(iter, name)?;
        let crypto_type = Type::from_iter(iter, name)?;
        Ok(CryptoBlock {
            name: name.to_string(),
            driver,
            module,
            priority,
            ref_count,
            self_test,
            internal,
            fips_enabled,
            crypto_type,
        })
    }
}

/// Potential results for selftest.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum SelfTest {
    Passed,
    Unknown,
}

impl TryFrom<&str> for SelfTest {
    type Error = ProcError;

    fn try_from(value: &str) -> Result<Self, ProcError> {
        Ok(match value {
            "passed" => Self::Passed,
            "unknown" => Self::Unknown,
            _ => {
                return Err(build_internal_error!(format!(
                    "Could not recognise self test string {value}"
                )))
            }
        })
    }
}

/// Enumeration of potential types and their associated data. Unknown at end to catch unrecognised types.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub enum Type {
    /// Symmetric Key Cipher
    Skcipher(Skcipher),
    /// Single Block Cipher
    Cipher(Cipher),
    /// Synchronous Hash
    Shash(Shash),
    /// Asynchronous Hash
    Ahash(Ahash),
    /// Authenticated Encryption with Associated Data
    Aead(Aead),
    /// Random Number Generator
    Rng(Rng),
    /// Test algorithm
    Larval(Larval),
    /// Synchronous Compression
    Scomp,
    /// General Compression
    Compression,
    /// Asymmetric Cipher
    AkCipher,
    /// Key-agreement Protocol Primitive
    Kpp,
    /// Signature
    Sig,
    /// Unrecognised type, associated data collected in to a hash map
    Unknown(Unknown),
}

impl Type {
    fn from_iter<T: Iterator<Item = Result<String, std::io::Error>>>(
        iter: &mut Peekable<T>,
        name: &str,
    ) -> ProcResult<Self> {
        let type_name = parse_line(iter, "type", name)?;
        Ok(match type_name.as_str() {
            "skcipher" => Self::Skcipher(Skcipher::parse(iter, name)?),
            "cipher" => Self::Cipher(Cipher::parse(iter, name)?),
            "shash" => Self::Shash(Shash::parse(iter, name)?),
            "scomp" => Self::Scomp,
            "compression" => Self::Compression,
            "akcipher" => Self::AkCipher,
            "kpp" => Self::Kpp,
            "ahash" => Self::Ahash(Ahash::parse(iter, name)?),
            "aead" => Self::Aead(Aead::parse(iter, name)?),
            "rng" => Self::Rng(Rng::parse(iter, name)?),
            "larval" => Self::Larval(Larval::parse(iter, name)?),
            "sig" => Self::Sig,
            unknown_name => Self::Unknown(Unknown::parse(iter, unknown_name)),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Skcipher {
    pub async_capable: bool,
    pub block_size: usize,
    pub min_key_size: usize,
    pub max_key_size: usize,
    pub iv_size: usize,
    pub chunk_size: usize,
    pub walk_size: usize,
}

impl Skcipher {
    fn parse<T: Iterator<Item = Result<String, std::io::Error>>>(iter: &mut T, name: &str) -> ProcResult<Self> {
        let async_capable = parse_bool(iter, "async", name)?;
        let block_size = from_str!(usize, &parse_line(iter, "blocksize", name)?);
        let min_key_size = from_str!(usize, &parse_line(iter, "min keysize", name)?);
        let max_key_size = from_str!(usize, &parse_line(iter, "max keysize", name)?);
        let iv_size = from_str!(usize, &parse_line(iter, "ivsize", name)?);
        let chunk_size = from_str!(usize, &parse_line(iter, "chunksize", name)?);
        let walk_size = from_str!(usize, &parse_line(iter, "walksize", name)?);
        Ok(Self {
            async_capable,
            block_size,
            min_key_size,
            max_key_size,
            iv_size,
