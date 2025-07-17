use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use fs2::FileExt;

use colored::Color;
use crate::color::DynamicColorize;

/// Manages invoice sequence numbers and their associated dates.
///
/// This struct handles reading from and writing to an index file, ensuring
/// that sequence numbers are unique and persistent across application runs.
pub struct Index {
    file_path: PathBuf,
    sequences: HashMap<u32, Vec<String>>,
    lock_file: File, // Held for exclusive lock
}

impl Index {
    /// Creates a new `Index` instance, loading existing sequences from the specified file.
    ///
    /// It acquires an exclusive lock on the index file to prevent concurrent access.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the index file.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the file cannot be opened, locked, or read.
    pub fn new(file_path: &Path) -> Result<Self, io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(file_path)?;

        file.lock_exclusive()?;

        let mut index = Index {
            file_path: file_path.to_path_buf(),
            sequences: HashMap::new(),
            lock_file: file,
        };

        index.load()?;
        Ok(index)
    }

    // Loads sequence numbers and their associated dates from the index file.
    fn load(&mut self) -> Result<(), io::Error> {
        self.sequences.clear();
        let file = BufReader::new(File::open(&self.file_path)?);
        for line in file.lines() {
            let line = line?;
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                if let Ok(sequence) = parts[0].parse::<u32>() {
                    let dates: Vec<String> = parts[1].split_whitespace().map(|s| s.to_string()).collect();
                    self.sequences.insert(sequence, dates);
                } else {
                    tracing::warn!("Invalid sequence number in index file: {}", line.err_colored(Color::Yellow));
                }
            } else {
                tracing::warn!("Invalid line in index file: {}", line.err_colored(Color::Yellow));
            }
        }
        Ok(())
    }

    /// Saves the current state of sequence numbers and dates to the index file.
    ///
    /// It writes to a temporary file first and then renames it to ensure data integrity.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the file cannot be written to or renamed.
    pub fn save(&self) -> Result<(), io::Error> {
        let temp_path = self.file_path.with_extension("tmp");
        let mut temp_file = File::create(&temp_path)?;

        tracing::debug!("temp index: {}", temp_path.display());

        let mut sorted_sequences: Vec<(&u32, &Vec<String>)> = self.sequences.iter().collect();
        sorted_sequences.sort_by_key(|(seq, _)| *seq);

        for (sequence, dates) in sorted_sequences {
            tracing::debug!("INDEX {} {}", sequence, dates.join(" "));
            writeln!(temp_file, "{} {}", sequence, dates.join(" "))?;
        }

        fs::rename(&temp_path, &self.file_path)?;
        Ok(())
    }

    /// Adds a new sequence number with associated dates to the index.
    ///
    /// # Arguments
    ///
    /// * `sequence` - The sequence number to add.
    /// * `dates` - A slice of date strings associated with the sequence.
    ///
    /// # Returns
    ///
    /// The added sequence number.
    pub fn add_sequence(&mut self, sequence: u32, dates: &[String]) -> u32 {
        self.sequences.insert(sequence, dates.to_vec());
        sequence
    }

    /// Finds an existing sequence number for a given set of dates, or generates a new one.
    ///
    /// If a matching set of dates is found, its sequence number is returned.
    /// Otherwise, a new sequence number (max existing + 1) is generated and associated with the dates.
    ///
    /// # Arguments
    ///
    /// * `dates` - A slice of date strings to search for or associate with a new sequence.
    ///
    /// # Returns
    ///
    /// The found or newly generated sequence number.
    pub fn find_sequence(&mut self, dates: &[String]) -> u32 {
        let mut sorted_input_dates = dates.to_vec();
        sorted_input_dates.sort();

        for (seq, stored_dates) in &self.sequences {
            let mut sorted_stored_dates = stored_dates.clone();
            sorted_stored_dates.sort();
            if sorted_stored_dates == sorted_input_dates {
                return *seq;
            }
        }
        // If not found, generate next sequence number
        let seq = self.sequences.keys().max().map_or(1, |&max_seq| max_seq + 1);
        // and add it to the list
        self.sequences.insert(seq, sorted_input_dates);
        seq
    }
}

impl Drop for Index {
    /// Releases the exclusive lock on the index file when the `Index` instance is dropped.
    fn drop(&mut self) {
      if let Err(e) = fs2::FileExt::unlock(&self.lock_file) {
          tracing::error!("Failed to unlock index file: {}",
              format!("{}", e).err_colored(Color::Red));
      }
    }
}
