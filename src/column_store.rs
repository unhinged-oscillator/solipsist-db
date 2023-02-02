use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::SystemTime;

use crate::column_value::ColumnValue;

// # person table
// ("Name", "Date of birth", "Waist Size", "token")
// ("John", "31.02.1991", 32, [0x00, 0x00])
// ("Mary", "31.10.1951", 45, [0x00, 0x00])
// ("Bob", "31.01.1991", 38, [0x00, 0x00])

// ("Name" "John", "Mary", "Bob")

struct ColumnStore {
    columns: BTreeMap<String, ColumnFile>,
}

impl ColumnStore {
    fn open<P: AsRef<Path>>(path: P) -> std::io::Result<ColumnStore> {
        let mut columns = BTreeMap::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let column_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_owned();
            let column_file = ColumnFile::open(path)?;
            // columns.insert(column_name, column_file);
        }
        Ok(ColumnStore { columns })
    }

    fn insert(
        &mut self,
        column_name: &str,
        value: ColumnValue,
        timestamp: (SystemTime, u64),
    ) -> std::io::Result<()> {
        let column_file = self
            .columns
            .entry(column_name.to_owned())
            .or_insert_with(|| ColumnFile::create(column_name));
        // column_file.write(value)
        Ok(())
    }

    fn query(&self, column_name: &str, start: u64, end: u64) -> Vec<u64> {
        let column_file = self
            .columns
            .get(column_name)
            .unwrap_or_else(|| panic!("Column not found: {}", column_name));
        // column_file.query(start, end)
        vec![]
    }
}
struct ColumnFile {
    writer: BufWriter<File>,
    reader: BufReader<File>,
}

impl ColumnFile {
    fn open<P: AsRef<Path>>(path: P) -> std::io::Result<ColumnFile> {
        let file = File::open(path)?;
        let reader = BufReader::new(file.try_clone()?);
        let writer = BufWriter::new(file);
        Ok(ColumnFile { reader, writer })
    }

    fn create(name: &str) -> ColumnFile {
        let path = Path::new(name);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        let reader = BufReader::new(file.try_clone().unwrap());
        let writer = BufWriter::new(file);
        ColumnFile { reader, writer }
    }

    fn write(&mut self, value: u64) -> std::io::Result<()> {
        self.writer.write_all(&value.to_le_bytes())?;
        self.writer.flush()
    }

    fn query(&mut self, start: u64, end: u64) -> Vec<u64> {
        let mut values = Vec::new();
        self.reader.seek(SeekFrom::Start(start * 8)).unwrap();
        let mut buf = [0; 8];
        while self.reader.read_exact(&mut buf).is_ok() {
            let value = u64::from_le_bytes(buf);
            if value > end {
                break;
            }
            values.push(value);
        }
        values
    }
}
