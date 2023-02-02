use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct WriteAheadLog {
    writer: BufWriter<File>,
}

impl WriteAheadLog {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<WriteAheadLog> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let writer = BufWriter::new(file);
        Ok(WriteAheadLog { writer })
    }

    pub fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()
    }
}

pub struct WriteAheadLogReader {
    reader: BufReader<File>,
}

impl WriteAheadLogReader {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<WriteAheadLogReader> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(WriteAheadLogReader { reader })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }

    pub fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}
