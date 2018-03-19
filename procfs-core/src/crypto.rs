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
