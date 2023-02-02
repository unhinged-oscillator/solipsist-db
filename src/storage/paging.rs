use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Result, Write};
use std::path::Path;

use crate::byte_encoder::{ByteDecoder, ByteEncoder};

use super::b_tree::{BTreePageType, BtreeNode};

#[repr(C, packed)]
struct PageHeader {
    page_type: BTreePageType,
    offset: u16,
    n_cells: u16,
    cell_offset: u16,
    right_pointer: u32,
}

impl PageHeader {
    fn from_bytes(bytes: &[u8]) -> Result<PageHeader> {
        let mut reader = ByteDecoder::new(BufReader::new(bytes));
        Ok(PageHeader {
            page_type: BTreePageType::from(reader.read_u8()?),
            offset: reader.read_u16()?,
            n_cells: reader.read_u16()?,
            cell_offset: reader.read_u16()?,
            right_pointer: reader.read_u32()?,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut writer = ByteEncoder::new(vec![]);
        writer.write_u16(self.offset)?;
        writer.write_u16(self.n_cells)?;
        writer.write_u16(self.cell_offset)?;
        writer.write_u32(self.right_pointer)?;
        Ok(writer.inner)
    }
}

const PAGE_SIZE: usize = 4096;

pub struct Page {
    data: [u8; PAGE_SIZE],
    btree_keys: Vec<u32>,
    checksum: Vec<u8>,
    header: PageHeader,
}

impl Page {
    // fn new() -> Self {
    //     Page {
    //         data: [0; PAGE_SIZE],
    //         btree_keys: Vec::new(),
    //         checksum: vec![],
    //     }
    // }

    pub fn write_to_file(&self) {}
}
