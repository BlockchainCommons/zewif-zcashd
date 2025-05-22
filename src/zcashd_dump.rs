use anyhow::{Context, Result, bail};
use hex::ToHex;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use super::BDBDump;
use crate::{parse, parser::prelude::*};
use zewif::Data;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DBKey {
    pub keyname: String,
    pub data: Data,
}

impl std::fmt::Display for DBKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.data.is_empty() {
            write!(f, "{}", self.keyname)
        } else {
            write!(f, "{}-{}", self.keyname, hex::encode(&self.data))
        }
    }
}

impl std::fmt::Debug for DBKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DBKey({})", self)
    }
}

impl DBKey {
    pub fn new(keyname: impl Into<String>, data: impl AsRef<Data>) -> Self {
        Self {
            keyname: keyname.into(),
            data: data.as_ref().clone(),
        }
    }

    pub fn parse_data(key_data: &Data) -> Result<Self> {
        let mut parser = Parser::new(&key_data);
        let keyname = parse!(&mut parser, "keyname")?;
        let data = parser.rest();
        parser.check_finished()?;
        Ok(Self { keyname, data })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DBValue(Data);

impl DBValue {
    pub fn new(data: Data) -> Self {
        Self(data)
    }

    pub fn as_data(&self) -> &Data {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for DBValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl AsRef<Data> for DBValue {
    fn as_ref(&self) -> &Data {
        &self.0
    }
}

impl AsRef<[u8]> for DBValue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct ZcashdDump {
    records: HashMap<DBKey, DBValue>,
    keys_by_keyname: HashMap<String, HashSet<DBKey>>,
}

impl ZcashdDump {
    pub fn from_bdb_dump(berkeley_dump: &BDBDump, strict: bool) -> Result<Self> {
        let mut records: HashMap<DBKey, DBValue> = HashMap::new();
        let mut keys_by_keyname: HashMap<String, HashSet<DBKey>> = HashMap::new();

        for (key_data, value_data) in &berkeley_dump.data_records {
            match DBKey::parse_data(key_data) {
                Ok(key) => {
                    let value = DBValue::new(value_data.clone());
                    records.insert(key.clone(), value.clone());

                    let keyname = key.keyname.to_string();
                    let keyname_keys = keys_by_keyname.entry(keyname).or_default();
                    keyname_keys.insert(key);
                }
                Err(e) if !strict => {
                    eprintln!(
                        "Unable to parse database key {}: {}",
                        key_data.encode_hex::<String>(),
                        e
                    );
                }
                err => {
                    err?;
                }
            }
        }

        Ok(ZcashdDump {
            records,
            keys_by_keyname,
        })
    }

    pub fn records(&self) -> &HashMap<DBKey, DBValue> {
        &self.records
    }

    pub fn value_for_key(&self, key: &DBKey) -> Result<&DBValue> {
        match self.records.get(key) {
            Some(value) => Ok(value),
            None => bail!("No record found for key: {}", key),
        }
    }

    pub fn key_for_keyname(&self, keyname: &str) -> DBKey {
        DBKey::new(keyname.to_string(), Data::new())
    }

    pub fn value_for_keyname(&self, keyname: &str) -> Result<&DBValue> {
        let key = self.key_for_keyname(keyname);
        self.value_for_key(&key)
            .context(format!("No record found for keyname: {}", keyname))
    }

    pub fn has_value_for_keyname(&self, keyname: &str) -> bool {
        let key = self.key_for_keyname(keyname);
        self.records.contains_key(&key)
    }

    #[allow(dead_code)]
    pub fn keys_by_keyname(&self) -> &HashMap<String, HashSet<DBKey>> {
        &self.keys_by_keyname
    }

    pub fn records_for_keyname(&self, keyname: &str) -> Result<HashMap<DBKey, DBValue>> {
        let keys = self
            .keys_by_keyname
            .get(keyname)
            .context(format!("No records found for keyname: {}", keyname))?;
        let mut records = HashMap::new();
        for key in keys {
            let value = self.value_for_key(key)?;
            records.insert(key.clone(), value.clone());
        }
        Ok(records)
    }

    pub fn has_keys_for_keyname(&self, keyname: &str) -> bool {
        self.keys_by_keyname.contains_key(keyname)
    }

    pub fn record_for_keyname(&self, keyname: &str) -> Result<(DBKey, DBValue)> {
        let keys = self
            .keys_by_keyname
            .get(keyname)
            .context(format!("No records found for keyname: {}", keyname))?;
        if keys.len() != 1 {
            bail!("Expected exactly one record for keyname: {}", keyname);
        }
        match keys.iter().next() {
            Some(key) => {
                let value = self.value_for_key(key)?;
                Ok((key.clone(), value.clone()))
            }
            None => bail!("No record found for keyname: {}", keyname),
        }
    }

    fn sorted_key_names(&self) -> Vec<String> {
        let mut keynames: Vec<String> = self.keys_by_keyname.keys().cloned().collect();
        keynames.sort();
        keynames
    }

    pub fn keyname_summary(&self) -> String {
        let mut output = String::new();
        for keyname in self.sorted_key_names() {
            let keys = self.keys_by_keyname.get(&keyname).unwrap();
            let mut min_value_size: usize = usize::MAX;
            let mut max_value_size: usize = 0;
            for key in keys.clone() {
                let value = self.records.get(&key).unwrap();
                min_value_size = min_value_size.min(value.len());
                max_value_size = max_value_size.max(value.len());
            }

            let s = if min_value_size == max_value_size {
                format!("{}", min_value_size)
            } else {
                format!("{}...{}", min_value_size, max_value_size)
            };
            writeln!(output, "{}: {} ({})", keyname, keys.len(), s).unwrap();
        }
        output
    }

    #[allow(dead_code)]
    pub fn dump_keys(&self) -> String {
        let mut output = String::new();
        for keyname in self.sorted_key_names() {
            writeln!(output, "{}", keyname).unwrap();
            let mut keys: Vec<DBKey> = self
                .keys_by_keyname
                .get(&keyname)
                .unwrap()
                .iter()
                .cloned()
                .collect();
            keys.sort();
            for key in keys {
                writeln!(output, "    {}", key).unwrap();
                let value = self.records.get(&key).unwrap();
                writeln!(output, "        {}: {}", value.len(), hex::encode(value)).unwrap();
                writeln!(output).unwrap();
            }
        }
        output
    }
}
