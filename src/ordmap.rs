use std::collections::hash_map::{DefaultHasher, HashMap};
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Index;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct OrdHashMap<K, V, H = DefaultHasher> {
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
    map: HashMap<u64, NonNull<Node<K, V>>>,
    _hasher: PhantomData<H>,
}

#[derive(Debug)]
struct Node<K, V> {
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
    key: K,
    value: V,
}

/// Referential iterator
pub struct Iter<'a, K, V>(Option<NonNull<Node<K, V>>>, PhantomData<&'a ()>);

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.0 {
            self.0 = unsafe { (*node.as_ptr()).next };
            let key = unsafe { &(*node.as_ptr()).key };
            let value = unsafe { &(*node.as_ptr()).value };
            Some((key, value))
        } else {
            None
        }
    }
}

/// Item iterator
pub struct IntoIter<K, V, H>(OrdHashMap<K, V, H>);

impl<K, V, H> Iterator for IntoIter<K, V, H> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.0.head {
            let node_ptr = node.as_ptr();
            self.0.head = unsafe { (*node_ptr).next };
            let key = unsafe { *Box::from_raw(&mut (*node_ptr).key) };
            let value = unsafe { *Box::from_raw(&mut (*node_ptr).value) };
            Some((key, value))
        } else {
            None
        }
    }
}

impl<K: Eq + Hash, V, H> Default for OrdHashMap<K, V, H> {
    fn default() -> Self {
        OrdHashMap {
            head: None,
            tail: None,
            map: HashMap::new(),
            _hasher: PhantomData::default(),
        }
    }
}

impl<K: Eq + Hash + Clone + Debug, V: Clone + Debug + Eq, H: Hasher + Default> Clone
    for OrdHashMap<K, V, H>
{
    fn clone(&self) -> Self {
        let mut new_list = OrdHashMap::default();
        for (key, value) in self.iter() {
            new_list.insert(key.clone(), value.clone());
        }
        new_list
    }
}

impl<K: Eq + Hash + Debug, V: Debug, H> Debug for OrdHashMap<K, V, H> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{")?;
        let mut is_first = true;
        for (key, value) in self.iter() {
            if is_first {
                is_first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{:?}: {:?}", key, value)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<K: Eq + Hash + Debug, V: Debug + Eq, H: Hasher + Default> Index<&K> for OrdHashMap<K, V, H> {
    type Output = V;
    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<K, V, H> Drop for OrdHashMap<K, V, H> {
    fn drop(&mut self) {
        for ptr in self.map.values_mut() {
            let boxed = unsafe { Box::from_raw(ptr) };
        }
    }
}

impl<K: Eq + Hash + Debug, V: Debug + Eq, H: Hasher + Default> OrdHashMap<K, V, H> {
    /// Gets the item using its key
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut h = H::default();
        key.hash(&mut h);
        let hash = h.finish();

        if let Some(node) = self.map.get(&hash) {
            Some(&unsafe { node.as_ref() }.value)
        } else {
            None
        }
    }

    /// Removes the specified item
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut h = H::default();
        key.hash(&mut h);
        let hash = h.finish();

        let result = if let Some(node) = self.map.remove(&hash) {
            let node = unsafe { Box::from_raw(node.as_ptr()) };

            // set the previous's next to be this next
            if let Some(mut prev) = node.prev {
                unsafe { prev.as_mut() }.next = node.next;
            }
            Some(node.value)
        } else {
            None
        };

        // debug_assert!(self.check_validity("remove::end"), "must be valid");

        result
    }

    /// Inserts the specified item into the list
    ///
    /// The inserted item will always be the last regardless of whether or not the key existed previously.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // debug_assert!(self.check_validity("insert::begin"), "must be valid");

        let mut h = H::default();
        key.hash(&mut h);
        let hash = h.finish();

        let old_value = self.remove(&key);

        let node = Node {
            prev: None,
            next: None,
            key,
            value,
        };
        let boxed_node = Box::new(node);
        let raw_boxed_node = Box::into_raw(boxed_node);
        let mut node = unsafe { NonNull::new_unchecked(raw_boxed_node) };

        if let Some(ref mut tail) = self.tail {
            (*unsafe { node.as_mut() }).prev = Some(*tail);
            (*unsafe { tail.as_mut() }).next = Some(node);
        }

        if let None = self.head {
            self.head = Some(node);
        }
        self.tail = Some(node);
        self.map.insert(hash, node);
        panic!("Boxed node: {:?}", unsafe { node.as_ref() });

        debug_assert!(self.check_validity("insert::end"), "must be valid");

        old_value
    }

    #[cfg(debug_assertions)]
    fn check_validity(&self, from: impl AsRef<str>) -> bool {
        let mut curr = self.head;
        let mut new_map = self.map.clone();
        let len = new_map.len();

        for i in 0..len {
            // make sure we're still there
            debug_assert!(curr.is_some(), "Linked list too short ({} vs. expected {}).", i, len);
            let curr2 = curr.unwrap();
            let node = unsafe { curr2.as_ref() };

            let mut h = H::default();
            node.key.hash(&mut h);
            let hash = h.finish();

            let removed = new_map.remove(&hash);
            debug_assert!(removed.is_some(), "Linked list node {:?} is not in the hashmap.", node);
            let node_ptr = removed.unwrap();

            let hnode = unsafe { node_ptr.as_ref() };
            debug_assert!(hnode.value == node.value, "Linked list node {:?} doesn't have a matching value", node);
            debug_assert!(false, "Got key {:?} and value {:?}", node.key, node.value);
        }

        debug_assert!(new_map.len() == 0, "New hash map has too many elements: {:?}", new_map);
        debug_assert!(false, "wtf {}: {:?}", from.as_ref(), self);
        true
    }
}

impl<K: Eq + Hash + Debug, V: Debug + Eq> OrdHashMap<K, V, DefaultHasher> {
    /// Creates a new OrdHashMap<T>
    pub fn new() -> Self {
        OrdHashMap::default()
    }
}

impl<K: Eq + Hash, V, H> OrdHashMap<K, V, H> {
    /// Creates an iterator for the list
    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter(self.head, PhantomData::default())
    }

    /// Creates a consuming iterator for the list
    pub fn into_iter(self) -> IntoIter<K, V, H> {
        IntoIter(self)
    }

    /// Get the length of the map
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::OrdHashMap;
    use std::cell::RefCell;
    use std::collections::HashMap;

    // use proptest::{
    //     arbitrary::any,
    //     collection::{hash_map, SizeRange},
    //     prelude::*,
    //     sample::Index,
    // };

    // proptest! {
    //     #[test]
    //     fn test_hashmap_equivalence(map in hash_map(any::<u32>(), any::<u32>(), SizeRange::default())) {
    //         let mut ordmap = OrdHashMap::new();
    //         for (key, value) in map.iter() {
    //             ordmap.insert(*key, *value);
    //         }
    //         prop_assert!(ordmap.iter().eq(map.iter()));
    //         panic!("bogue {:?}", map);
    //     }
    // }

    #[test]
    fn test_insert() {
        let mut m = OrdHashMap::new();
        assert_eq!(m.len(), 0);
        assert!(m.insert(1, 2).is_none());
        assert_eq!(m.len(), 1);
        assert!(m.insert(2, 4).is_none());
        assert_eq!(m.len(), 2);
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert_eq!(*m.get(&2).unwrap(), 4);
    }

    #[test]
    fn test_clone() {
        let mut m = HashMap::new();
        assert_eq!(m.len(), 0);
        assert!(m.insert(1, 2).is_none());
        assert_eq!(m.len(), 1);
        assert!(m.insert(2, 4).is_none());
        assert_eq!(m.len(), 2);
        let m2 = m.clone();
        assert_eq!(*m2.get(&1).unwrap(), 2);
        assert_eq!(*m2.get(&2).unwrap(), 4);
        assert_eq!(m2.len(), 2);
    }

    thread_local! { static DROP_VECTOR: RefCell<Vec<i32>> = RefCell::new(Vec::new()) }

    #[derive(Hash, PartialEq, Eq)]
    struct Droppable {
        k: usize,
    }

    impl Droppable {
        fn new(k: usize) -> Droppable {
            DROP_VECTOR.with(|slot| {
                slot.borrow_mut()[k] += 1;
            });

            Droppable { k }
        }
    }

    impl Drop for Droppable {
        fn drop(&mut self) {
            DROP_VECTOR.with(|slot| {
                slot.borrow_mut()[self.k] -= 1;
            });
        }
    }

    impl Clone for Droppable {
        fn clone(&self) -> Self {
            Droppable::new(self.k)
        }
    }

    #[test]
    fn test_drops() {
        DROP_VECTOR.with(|slot| {
            *slot.borrow_mut() = vec![0; 200];
        });

        {
            let mut m = HashMap::new();

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 0);
                }
            });

            for i in 0..100 {
                let d1 = Droppable::new(i);
                let d2 = Droppable::new(i + 100);
                m.insert(d1, d2);
            }

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 1);
                }
            });

            for i in 0..50 {
                let k = Droppable::new(i);
                let v = m.remove(&k);

                assert!(v.is_some());

                DROP_VECTOR.with(|v| {
                    assert_eq!(v.borrow()[i], 1);
                    assert_eq!(v.borrow()[i + 100], 1);
                });
            }

            DROP_VECTOR.with(|v| {
                for i in 0..50 {
                    assert_eq!(v.borrow()[i], 0);
                    assert_eq!(v.borrow()[i + 100], 0);
                }

                for i in 50..100 {
                    assert_eq!(v.borrow()[i], 1);
                    assert_eq!(v.borrow()[i + 100], 1);
                }
            });
        }

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 0);
            }
        });
    }

    #[test]
    fn test_into_iter_drops() {
        DROP_VECTOR.with(|v| {
            *v.borrow_mut() = vec![0; 200];
        });

        let hm = {
            let mut hm = HashMap::new();

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 0);
                }
            });

            for i in 0..100 {
                let d1 = Droppable::new(i);
                let d2 = Droppable::new(i + 100);
                hm.insert(d1, d2);
            }

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 1);
                }
            });

            hm
        };

        // By the way, ensure that cloning doesn't screw up the dropping.
        drop(hm.clone());

        {
            let mut half = hm.into_iter().take(50);

            DROP_VECTOR.with(|v| {
                for i in 0..200 {
                    assert_eq!(v.borrow()[i], 1);
                }
            });

            for _ in half.by_ref() {}

            DROP_VECTOR.with(|v| {
                let nk = (0..100).filter(|&i| v.borrow()[i] == 1).count();

                let nv = (0..100).filter(|&i| v.borrow()[i + 100] == 1).count();

                assert_eq!(nk, 50);
                assert_eq!(nv, 50);
            });
        };

        DROP_VECTOR.with(|v| {
            for i in 0..200 {
                assert_eq!(v.borrow()[i], 0);
            }
        });
    }

    #[test]
    fn test_empty_remove() {
        let mut m: HashMap<i32, bool> = HashMap::new();
        assert_eq!(m.remove(&0), None);
    }

    // #[test]
    // fn test_empty_entry() {
    //     let mut m: HashMap<i32, bool> = HashMap::new();
    //     match m.entry(0) {
    //         Occupied(_) => panic!(),
    //         Vacant(_) => {}
    //     }
    //     assert!(*m.entry(0).or_insert(true));
    //     assert_eq!(m.len(), 1);
    // }

    #[test]
    fn test_empty_iter() {
        let mut m: HashMap<i32, bool> = HashMap::new();
        assert_eq!(m.drain().next(), None);
        assert_eq!(m.keys().next(), None);
        assert_eq!(m.values().next(), None);
        assert_eq!(m.values_mut().next(), None);
        assert_eq!(m.iter().next(), None);
        assert_eq!(m.iter_mut().next(), None);
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());
        assert_eq!(m.into_iter().next(), None);
    }
}
