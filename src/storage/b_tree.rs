use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::{BufReader, Read, Result};

use crate::byte_encoder::{ByteDecoder, ByteEncoder};

const ORDER: usize = 4;

pub enum BTreePageType {
    InteriorTable,
    LeafTable,
    InteriorIndex,
    LeafIndex,
}

impl From<u8> for BTreePageType {
    fn from(value: u8) -> Self {
        match value {
            0x02 => BTreePageType::InteriorIndex,
            0x05 => BTreePageType::InteriorTable,
            0x0a => BTreePageType::LeafIndex,
            0x0d => BTreePageType::LeafTable,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct KeysIterator<'a> {
    results: VecDeque<u64>,
    queue: VecDeque<&'a Node>,
}

impl<'a> Default for KeysIterator<'a> {
    fn default() -> Self {
        KeysIterator {
            results: VecDeque::new(),
            queue: VecDeque::new(),
        }
    }
}

impl<'a> Iterator for KeysIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.results.is_empty() {
            return self.results.pop_front();
        }
        loop {
            if let Some(node) = self.queue.pop_front() {
                match &node {
                    Node::Leaf { values, .. } => {
                        for value in values.iter() {
                            self.results.push_back(value.key);
                        }

                        if !values.is_empty() {
                            break;
                        }
                    }
                    Node::Internal { children, .. } => {
                        for child in children.iter() {
                            self.queue.push_back(child);
                        }
                    }
                }
            } else {
                break;
            }
        }
        if !self.results.is_empty() {
            return self.results.pop_front();
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyValuePair {
    pub key: u64,
    pub value: Vec<u8>,
}

impl KeyValuePair {
    pub fn new(key: u64, value: Vec<u8>) -> Self {
        KeyValuePair { key, value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Node {
    Internal {
        keys: Vec<u64>,
        children: Vec<Node>,
    },

    Leaf {
        keys: Vec<u64>,
        values: Vec<KeyValuePair>,
    },
}

impl Node {
    fn new_leaf() -> Self {
        Node::Leaf {
            keys: vec![],
            values: vec![],
        }
    }

    fn new_internal() -> Self {
        Node::Internal {
            keys: vec![],
            children: vec![],
        }
    }

    pub fn is_full(&self) -> bool {
        match self {
            Node::Leaf { keys, .. } => keys.len() > 2 * ORDER - 1,
            Node::Internal { children, .. } => children.len() > 2 * ORDER,
        }
    }

    // Split a node into two new nodes when it has more than `ORDER` keys
    fn split(&mut self) -> (Node, u64, Node) {
        let mid = match self {
            Node::Leaf { values, .. } => values.len() / 2 + 1,
            Node::Internal { keys, .. } => keys.len() / 2,
        };
        match self {
            Node::Leaf { keys, values } => {
                let (keys, values) = (keys.clone(), values.clone());
                println!("SPLITTING AT {mid} | {keys:?} | {values:?}");
                let left = Node::Leaf {
                    keys: keys[..mid].to_vec(),
                    values: values[..mid].to_vec(),
                };
                let right = Node::Leaf {
                    keys: keys[mid..].to_vec(),
                    values: values[mid..].to_vec(),
                };
                (left, keys[mid - 1], right)
            }
            Node::Internal { keys, children } => {
                let (keys, children) = (keys.clone(), children.clone());
                let left = Node::Internal {
                    keys: keys[..mid].to_vec(),
                    children: children[..mid + 1].to_vec(),
                };
                let right = Node::Internal {
                    keys: keys[mid + 1..].to_vec(),
                    children: children[mid + 1..].to_vec(),
                };
                (left, keys[mid], right)
            }
        }
    }

    // Insert a key-value pair into the B+ tree
    fn insert(&mut self, key: u64, value: Vec<u8>) {
        match self {
            Node::Leaf { keys, values } => {
                match keys.binary_search(&key) {
                    Ok(pos) => panic!("key '{pos}' already exists"),
                    Err(pos) => {
                        keys.insert(pos, key);
                        values.insert(pos, KeyValuePair { key, value });
                        pos
                    }
                };

                // split leaf node if it has become too large
                if self.is_full() {
                    let (left, new_key, right) = self.split();

                    *self = Node::Internal {
                        keys: vec![new_key],
                        children: vec![left, right],
                    };
                }
            }
            Node::Internal { keys, children } => {
                let pos = match keys.binary_search(&key) {
                    Ok(_) => panic!("Duplicate key"),
                    Err(pos) => pos,
                };

                // call child to insert data
                let child = &mut children[pos];
                child.insert(key, value);
                if child.is_full() {
                    let (left, new_key, right) = child.split();
                    *child = left;
                    keys.insert(pos, new_key);
                    children.insert(pos + 1, right);
                    // return left;
                }
                // return self;
            }
        }
    }

    fn search_leaf(&self, key: u64) -> &Self {
        match &self {
            Node::Internal { keys, children } => match keys.binary_search(&key) {
                Ok(index) => children[index].search_leaf(key),
                Err(_) => panic!(),
            },
            Node::Leaf { .. } => self,
        }
    }

    // fn remove(&mut self, key: u64) -> Option<KeyValuePair> {
    //     let index = match self.keys.binary_search(&key) {
    //         Ok(i) => i,
    //         Err(i) => i,
    //     };

    //     // If the key is in the current node, remove it
    //     if index < self.keys.len() && self.keys[index] == key {
    //         // If this is a leaf node, remove the key-value pair directly
    //         if self.is_leaf() {
    //             let value = self.values.remove(index);
    //             self.keys.remove(index);
    //             return Some(value);
    //         }

    //         // If this is an internal node, replace the key with the maximum key in the left child
    //         let mut left_child = &mut self.children.as_mut().unwrap()[index];
    //         while let Some(ref mut child) = left_child.children {
    //             let len = child.len();
    //             left_child = &mut child[len - 1];
    //         }
    //         let replacement_key = left_child.keys.pop().unwrap();
    //         let replacement_value = left_child.values.pop().unwrap();
    //         self.keys[index] = replacement_key.clone();
    //         self.values[index] = replacement_value;
    //         return left_child.remove(replacement_key);
    //     }

    //     // If the key is not in this node, continue down the children
    //     if let Some(ref mut children) = self.children {
    //         let child = &mut children[index];
    //         return child.remove(key);
    //     }

    //     // If we reach here, the key does not exist in the btree
    //     None
    // }
}

#[derive(Debug)]
pub struct Btree {
    root: Node,
}

impl Btree {
    pub fn new() -> Self {
        Btree {
            root: Node::new_leaf(),
        }
    }

    pub fn search(&self, key: u64) -> Option<KeyValuePair> {
        self.search_node(&self.root, key)
    }

    /// search_node recursively searches a sub tree rooted at node for a key.
    fn search_node(&self, node: &Node, search: u64) -> Option<KeyValuePair> {
        match &node {
            Node::Internal { children, keys } => {
                let idx = keys.binary_search(&search).unwrap_or_else(|x| x);
                // Retrieve child page from disk and deserialize.
                let child_node = children.get(idx).unwrap();
                // let page = self.pager.get_page(child_offset)?;
                // let child_node = Node::try_from(page)?;
                self.search_node(child_node, search)
            }
            Node::Leaf { values, .. } => {
                if let Ok(idx) = values.binary_search_by_key(&search, |pair| pair.key) {
                    return Some(values[idx].clone());
                }
                None
                // Err(Error::KeyNotFound)
            } // Node::Unexpected => Err(Error::UnexpectedError),
        }
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.root.insert(key, value);
    }

    pub fn keys(&self) -> KeysIterator {
        let mut iterator = KeysIterator::default();
        iterator.queue.push_back(&self.root);
        iterator
    }

    // pub fn remove(&mut self, key: u64) -> Option<KeyValuePair> {
    //     self.root.remove(key)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_leaf_works() {
        let mut node = Node::Leaf {
            values: vec![
                KeyValuePair::new(1, "bar".as_bytes().to_vec()),
                KeyValuePair::new(2, "james".as_bytes().to_vec()),
                KeyValuePair::new(3, "grande".as_bytes().to_vec()),
            ],
            keys: vec![1, 2, 3],
        };

        let (left, mid, sibling) = node.split();
        println!("SPLIT LEAF {left:?} | {sibling:?}");
        assert_eq!(mid, 2);
        assert_eq!(
            left,
            Node::Leaf {
                values: vec![
                    KeyValuePair {
                        key: 1,
                        value: "bar".as_bytes().to_vec(),
                    },
                    KeyValuePair {
                        key: 2,
                        value: "james".as_bytes().to_vec()
                    }
                ],
                keys: vec![1, 2],
            }
        );
        assert_eq!(
            sibling,
            Node::Leaf {
                keys: vec![3],
                values: vec![KeyValuePair::new(3, "grande".as_bytes().to_vec())]
            }
        );
    }

    #[test]
    fn split_internal_works() {
        let mut node = Node::Internal {
            // vec![
            //     Offset(PAGE_SIZE),
            //     Offset(PAGE_SIZE * 2),
            //     Offset(PAGE_SIZE * 3),
            //     Offset(PAGE_SIZE * 4),
            // ],
            children: vec![
                Node::Leaf {
                    keys: vec![],
                    values: vec![KeyValuePair::new(1, "bar".as_bytes().to_vec())],
                },
                Node::Leaf {
                    keys: vec![],
                    values: vec![KeyValuePair::new(2, "james".as_bytes().to_vec())],
                },
                Node::Leaf {
                    keys: vec![],
                    values: vec![KeyValuePair::new(3, "grande".as_bytes().to_vec())],
                },
            ],
            keys: vec![
                1, // Key("foo bar".to_string()),
                2, // Key("lebron".to_string()),
                3, // Key("ariana".to_string()),
            ],
        };

        let (left, median, sibling) = node.split();
        println!("AFTER SPLIT {} -> {:?}", median, sibling);
        assert_eq!(median, 2);
        assert_eq!(
            left,
            Node::Internal {
                // vec![Offset(PAGE_SIZE), Offset(PAGE_SIZE * 2)],
                keys: vec![1],
                children: vec![
                    Node::Leaf {
                        keys: vec![],
                        values: vec![KeyValuePair::new(1, "bar".as_bytes().to_vec(),)],
                    },
                    Node::Leaf {
                        keys: vec![],
                        values: vec![KeyValuePair::new(2, "james".as_bytes().to_vec(),)],
                    }
                ]
            }
        );
        assert_eq!(
            sibling,
            Node::Internal {
                // vec![Offset(PAGE_SIZE * 3), Offset(PAGE_SIZE * 4)],
                keys: vec![3],
                children: vec![Node::Leaf {
                    keys: vec![],
                    values: vec![KeyValuePair::new(3, "grande".as_bytes().to_vec(),)],
                },],
            }
        );
    }

    #[test]
    fn test_insert() {
        let mut btree = Btree::new();

        // Insert key-value pairs
        btree.insert(1, "value1".as_bytes().to_vec());
        btree.insert(2, "value2".as_bytes().to_vec());
        btree.insert(3, "value3".as_bytes().to_vec());
        btree.insert(4, "value4".as_bytes().to_vec());
        btree.insert(5, "value5".as_bytes().to_vec());
        btree.insert(6, "value6".as_bytes().to_vec());
        btree.insert(7, "value7".as_bytes().to_vec());
        btree.insert(8, "value8".as_bytes().to_vec());
        btree.insert(9, "value9".as_bytes().to_vec());
        btree.insert(10, "value10".as_bytes().to_vec());
        btree.insert(11, "value11".as_bytes().to_vec());
        btree.insert(12, "value12".as_bytes().to_vec());

        println!("BTREE AFTER ALL {btree:?}");
        // Check the values are stored correctly
        assert_eq!(
            btree.search(1),
            Some(KeyValuePair {
                key: 1,
                value: "value1".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(2),
            Some(KeyValuePair {
                key: 2,
                value: "value2".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(3),
            Some(KeyValuePair {
                key: 3,
                value: "value3".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(4),
            Some(KeyValuePair {
                key: 4,
                value: "value4".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(5),
            Some(KeyValuePair {
                key: 5,
                value: "value5".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(6),
            Some(KeyValuePair {
                key: 6,
                value: "value6".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(7),
            Some(KeyValuePair {
                key: 7,
                value: "value7".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(8),
            Some(KeyValuePair {
                key: 8,
                value: "value8".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(9),
            Some(KeyValuePair {
                key: 9,
                value: "value9".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(10),
            Some(KeyValuePair {
                key: 10,
                value: "value10".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(11),
            Some(KeyValuePair {
                key: 11,
                value: "value11".as_bytes().to_vec()
            })
        );
        assert_eq!(
            btree.search(12),
            Some(KeyValuePair {
                key: 12,
                value: "value12".as_bytes().to_vec()
            })
        );

        // Check the values are ordered correctly
        let keys: Vec<u64> = btree.keys().collect();
        assert_eq!(keys, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    }

    // #[test]
    // fn test_remove() {
    //     let mut btree = Btree::new();

    //     // Insert key-value pairs
    //     btree.insert(1, "value1");
    //     btree.insert(2, "value2");
    //     btree.insert(3, "value3");
    //     btree.insert(4, "value4");
    //     btree.insert(5, "value5");
    //     btree.insert(6, "value6");
    //     btree.insert(7, "value7");
    //     btree.insert(8, "value8");

    //     // Remove some values
    //     btree.remove(3);
    //     btree.remove(4);
    //     btree.remove(8);

    //     // Check the values were removed correctly
    //     assert_eq!(btree.search(3), None);
    //     assert_eq!(btree.search(4), None);
    //     assert_eq!(btree.search(8), None);

    //     // Check the values are still ordered correctly
    //     let keys: Vec<&u32> = btree.keys().collect();
    //     assert_eq!(keys, vec![&1, &2, &5, &6, &7]);
    // }

    // #[test]
    // fn test_iteration() {
    //     let mut btree = Btree::new();

    //     // Insert key-value pairs
    //     btree.insert(1, "value1");
    //     btree.insert(2, "value2");
    //     btree.insert(3, "value3");
    //     btree.insert(4, "value4");
    //     btree.insert(5, "value5");
    //     btree.insert(6, "value6");
    //     btree.insert(7, "value7");
    //     btree.insert(8, "value8");

    //     // Check the values are iterated correctly
    //     let values: Vec<&str> = btree.values().map(|v| v.1).collect();
    //     assert_eq!(
    //         values,
    //         vec!["value1", "value2", "value3", "value4", "value5"]
    //     );
    // }
}
