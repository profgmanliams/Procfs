use crate::{expect, ProcResult};

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::BufRead};

/// Represents the data from `/proc/cpuinfo`.
///
/// The `fields` field stores the fields that are common among all CPUs.  The `cpus` field stores
/// CPU-specific info.
///
/// For common fields, there are methods that will return the data, converted to a more appropriate
/// data type.  These methods will all return `None` if the field doesn't exist, or is in some
/// unexpected format (in that case, you'll have to access the string data directly).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct CpuInfo {
    /// This stores fields that are common among all CPUs
    pub fields: HashMap<String, String>,
    pub cpus: Vec<HashMap<String, String>>,
}

impl crate::FromBufRead for CpuInfo {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut list = Vec::new();
        let mut map = Some(HashMap::new());

        // the first line of a cpu block must start with "processor"
        let mut found_first = false;

        for line in r.lines().flatten() {
            if !line.is_empty() {
                let mut s = line.split(':');
                let key = expect!(s.next());
                if !found_first && key.trim() == "processor" {
                    found_first = true;
                }
                if !found_first {
                    continue;
                }
                if let Some(value) = s.next() {
                    let key = key.trim().to_owned();
                    let value = value.trim().to_owned();

                    map.get_or_insert(HashMap::new()).insert(key, value);
                }
            } else if let Some(map) = map.take() {
                list.push(map);
                found_first = false;
            }
        }
        if let Some(map) = map.take() {
            list.push(map);
        }

        // find properties that are the same for all cpus
        assert!(!list.is_empty());

        let common_fields: Vec<String> = list[0]
            .iter()
            .filter_map(|(key, val)| {
                if list.iter().all(|map| map.get(key).map_or(false, |v| v == val)) {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut common_map = HashMap::new();
        for (k, v) in &list[0] {
