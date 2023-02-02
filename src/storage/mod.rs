mod b_tree;
mod paging;
use std::fs::File;

pub(crate) struct Storage {
    current_file: File,
}

impl Storage {}
