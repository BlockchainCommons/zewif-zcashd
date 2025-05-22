use std::{
    collections::HashMap,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Result, anyhow, bail};

use zewif::Data;

pub struct BDBDump {
    pub header_records: HashMap<String, String>,
    pub data_records: HashMap<Data, Data>,
}

impl BDBDump {
    pub fn from_file(filepath: &Path) -> Result<Self> {
        // Execute the `db_dump` utility
        let output = Command::new("db_dump")
            .arg(filepath)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| anyhow!("Error executing db_dump against path {}: {}", filepath.to_string_lossy(), e))?;

        // Check if db_dump executed successfully
        if !output.status.success() {
            bail!(
                "db_dump failed with status: {}\nError: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Convert the stdout to a string for parsing
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Initialize HashMaps to hold header and data records
        let mut header_records: HashMap<String, String> = HashMap::new();
        let mut data_records: HashMap<Data, Data> = HashMap::new();

        // Flag to indicate if we're past the header
        let mut in_data_section = false;

        // Temporary variable to hold the key
        let mut current_key: Option<Data> = None;

        let mut records_count = 0;

        // Iterate over each line of the db_dump output
        for line in stdout.lines() {
            let trimmed = line.trim();

            // Check for the end of the header section
            if trimmed == "HEADER=END" {
                in_data_section = true;
                continue;
            }

            // Parse header lines
            if !in_data_section {
                if let Some(eq_pos) = trimmed.find('=') {
                    let key = &trimmed[..eq_pos];
                    let value = &trimmed[eq_pos + 1..];
                    header_records.insert(key.to_string(), value.to_string());
                } else {
                    eprintln!("Invalid header line: {}", trimmed);
                }
                continue;
            }

            if line.starts_with("DATA=END") {
                break;
            }

            // Each data entry line starts with a space; remove it
            let hex_str = trimmed.trim_start_matches(' ');

            // Decode the hexadecimal string
            let bytes = Data::from_hex(hex_str)?;

            // Alternate between key and value
            if current_key.is_none() {
                current_key = Some(bytes);
            } else {
                let key = current_key.take().unwrap();
                let value = bytes;
                data_records.insert(key, value);
                records_count += 1;
            }
        }

        // Check if there was an unmatched key without a corresponding value
        if current_key.is_some() {
            bail!("Warning: Found a key without a corresponding value.");
        }

        if records_count != data_records.len() {
            bail!("Warning: Non-uniqueness in keys detected.");
        }

        Ok(BDBDump { header_records, data_records })
    }
}
