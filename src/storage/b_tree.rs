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
    queue: VecDeque<&'a BtreeNode>,
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
        if let Some(node) = self.queue.pop_front() {
            println!("ITERATOR IN QUEUE {:?}", node);
            match &node.inner {
                BTreeInner::Leaf(values) => {
                    for value in values.iter() {
                        self.results.push_back(value.key);
                    }
                }
                BTreeInner::Internal { keys, children } => {
                    for child in children.iter() {
                        self.queue.push_back(child);
                    }
                }
                _ => {}
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
pub struct BtreeNode {
    is_root: bool,
    inner: BTreeInner,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BTreeInner {
    Internal {
        keys: Vec<u64>,
        children: Vec<BtreeNode>,
    },
    Leaf(Vec<KeyValuePair>),
}

impl Default for BtreeNode {
    fn default() -> Self {
        BtreeNode {
            is_root: false,
            inner: BTreeInner::Leaf(vec![]),
        }
    }
}

impl BtreeNode {
    fn new_leaf() -> Self {
        BtreeNode {
            is_root: false,
            inner: BTreeInner::Leaf(vec![]),
        }
    }

    fn new_internal() -> Self {
        BtreeNode {
            is_root: false,
            inner: BTreeInner::Internal {
                keys: vec![],
                children: vec![],
            },
        }
    }

    // fn search(&self, key: u64) -> Option<&KeyValuePair> {
    //     match self.keys.binary_search(&key) {
    //         Ok(index) => Some(&self.values[index]),
    //         Err(index) => {
    //             if self.is_leaf() {
    //                 None
    //             } else {
    //                 self.children.as_ref().unwrap()[index].search(key)
    //             }
    //         }
    //     }
    // }

    // fn insert(&mut self, key: u64, value: KeyValuePair) {
    //     let index = match self.keys.binary_search(&key) {
    //         Ok(index) => index,
    //         Err(index) => index,
    //     };

    //     println!("INSERTING key [{key:?}] = [{value:?}] into {self:?}");

    //     if self.is_leaf() {
    //         self.keys.insert(index, key);
    //         self.values.insert(index, value);
    //     } else {
    //         let child = &mut self.children.as_mut().unwrap()[index];
    //         child.insert(key, value);
    //     }

    //     if self.keys.len() > 2 * ORDER - 1 {
    //         self.split();
    //     }
    // }

    /// split creates sibling node for either Internal Node Or
    fn split(&mut self, median: usize) -> (u64, BtreeNode) {
        // let median = ORDER;

        match self.inner {
            BTreeInner::Internal {
                ref mut children,
                ref mut keys,
            } => {
                // Populate siblings keys.
                println!("SPLITTING INTERNAL {:?}", keys);
                let mut sibling_keys = keys.split_off(median - 1);
                // Pop median key - to be added to the parent..
                let median_key = sibling_keys.remove(0);
                // Populate siblings children.
                let sibling_children = children.split_off(median);
                (
                    median_key,
                    BtreeNode {
                        inner: BTreeInner::Internal {
                            children: sibling_children,
                            keys: sibling_keys,
                        },
                        is_root: false,
                    },
                )
            }
            BTreeInner::Leaf(ref mut pairs) => {
                // Populate siblings pairs.
                let sibling_pairs = pairs.split_off(median);
                // Pop median key.
                let median_pair = pairs.get(median - 1).unwrap().clone();

                (
                    median_pair.key,
                    BtreeNode {
                        inner: BTreeInner::Leaf(sibling_pairs),
                        is_root: false,
                    },
                )
            }
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
    root: BtreeNode,
}

impl<'a> Btree {
    pub fn new() -> Self {
        Btree {
            root: BtreeNode::new_leaf(),
        }
    }

    // pub fn insert(&mut self, key: u64, value: KeyValuePair) {
    //     println!(
    //         "NEEDS NEW ROOT? {:?} == {}",
    //         self.root.keys.len(),
    //         2 * ORDER - 1
    //     );
    //     if self.root.keys.len() == 2 * ORDER - 1 {
    //         // let mut new_root = BtreeNode::new();
    //         // new_root.children = Some(vec![std::mem::take(&mut self.root)]);
    //         // println!("TOOK NEW ROOT {new_root:?}");
    //         // new_root.is_leaf = false;
    //         // self.root = new_root;
    //         self.root.split();
    //         println!("NOW SPLIT ROOT {:?}", self.root);
    //     }
    //     self.root.insert(key, value);
    // }

    // pub fn search(&self, key: K) -> Option<&V> {
    //     println!("ROOT {:?} -> {:?}", key, self);
    //     self.root.search(&key)
    // }

    // pub fn keys(&'a self) -> KeysIterator<'a> {
    //     let mut iterator = KeysIterator::default();
    //     iterator.queue.push_back(&self.root);
    //     iterator
    // }

    // pub fn remove(&mut self, key: u64) -> Option<KeyValuePair> {
    //     self.root.remove(key)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_leaf_works() {
        let mut node = BtreeNode {
            inner: BTreeInner::Leaf(vec![
                KeyValuePair::new(1, "bar".as_bytes().to_vec()),
                KeyValuePair::new(2, "james".as_bytes().to_vec()),
                KeyValuePair::new(3, "grande".as_bytes().to_vec()),
            ]),
            is_root: true,
        };

        let (median, sibling) = node.split(2);
        assert_eq!(median, 2);
        assert_eq!(
            node.inner,
            BTreeInner::Leaf(vec![
                KeyValuePair {
                    key: 1,
                    value: "bar".as_bytes().to_vec(),
                },
                KeyValuePair {
                    key: 2,
                    value: "james".as_bytes().to_vec()
                }
            ])
        );
        assert_eq!(
            sibling.inner,
            BTreeInner::Leaf(vec![KeyValuePair::new(3, "grande".as_bytes().to_vec())])
        );
    }

    #[test]
    fn split_internal_works() {
        let mut node = BtreeNode {
            inner: BTreeInner::Internal {
                // vec![
                //     Offset(PAGE_SIZE),
                //     Offset(PAGE_SIZE * 2),
                //     Offset(PAGE_SIZE * 3),
                //     Offset(PAGE_SIZE * 4),
                // ],
                children: vec![
                    BtreeNode {
                        inner: BTreeInner::Leaf(vec![KeyValuePair::new(
                            1,
                            "bar".as_bytes().to_vec(),
                        )]),
                        is_root: false,
                    },
                    BtreeNode {
                        inner: BTreeInner::Leaf(vec![KeyValuePair::new(
                            2,
                            "james".as_bytes().to_vec(),
                        )]),
                        is_root: false,
                    },
                    BtreeNode {
                        inner: BTreeInner::Leaf(vec![KeyValuePair::new(
                            3,
                            "grande".as_bytes().to_vec(),
                        )]),
                        is_root: false,
                    },
                ],
                keys: vec![
                    1, // Key("foo bar".to_string()),
                    2, // Key("lebron".to_string()),
                    3, // Key("ariana".to_string()),
                ],
            },
            is_root: true,
        };

        let (median, sibling) = node.split(2);
        println!("AFTER SPLIT {} -> {:?}", median, sibling);
        assert_eq!(median, 2);
        assert_eq!(
            node.inner,
            BTreeInner::Internal {
                // vec![Offset(PAGE_SIZE), Offset(PAGE_SIZE * 2)],
                keys: vec![1],
                children: vec![
                    BtreeNode {
                        inner: BTreeInner::Leaf(vec![KeyValuePair::new(
                            1,
                            "bar".as_bytes().to_vec(),
                        )]),
                        is_root: false,
                    },
                    BtreeNode {
                        inner: BTreeInner::Leaf(vec![KeyValuePair::new(
                            2,
                            "james".as_bytes().to_vec(),
                        )]),
                        is_root: false,
                    }
                ]
            }
        );
        assert_eq!(
            sibling.inner,
            BTreeInner::Internal {
                // vec![Offset(PAGE_SIZE * 3), Offset(PAGE_SIZE * 4)],
                keys: vec![3],
                children: vec![BtreeNode {
                    inner: BTreeInner::Leaf(vec![KeyValuePair::new(
                        3,
                        "grande".as_bytes().to_vec(),
                    )]),
                    is_root: false,
                },],
            }
        );
    }

    // #[test]
    // fn test_insert() {
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

    //     // Check the values are stored correctly
    //     assert_eq!(btree.search(1), Some(&"value1"));
    //     assert_eq!(btree.search(2), Some(&"value2"));
    //     assert_eq!(btree.search(3), Some(&"value3"));
    //     assert_eq!(btree.search(4), Some(&"value4"));
    //     assert_eq!(btree.search(5), Some(&"value5"));
    //     assert_eq!(btree.search(6), Some(&"value6"));
    //     assert_eq!(btree.search(7), Some(&"value7"));
    //     assert_eq!(btree.search(8), Some(&"value8"));

    //     // Check the values are ordered correctly
    //     let keys: Vec<&u32> = btree.keys().collect();
    //     assert_eq!(keys, vec![&1, &2, &3, &4, &5, &6, &7, &8]);
    // }

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
