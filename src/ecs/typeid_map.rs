use std::{any::TypeId, hash::{BuildHasherDefault, Hasher}};

use hashbrown::HashMap;

/// Inspired by HECS.
/// Since TypeId is already a hash, use a hasher that does nothing. 
/// Tested against a 256 item array with linear search and a 256 item hashmap.
pub(crate) type TypeIdMap<V> = HashMap<TypeId, V, BuildHasherDefault<TypeIdHasher>>;

#[test]
fn type_id_map_test() {
    let mut map: TypeIdMap<u32> = TypeIdMap::default();

    map.insert(TypeId::of::<u32>(), 2);

    assert_eq!(map.get(&TypeId::of::<u32>()), Some(&2));
}

/// This hasher does nothing.
/// Since TypeId is already a hash, we can just use that.
/// TypeId can be a u64 or u128 depending on ???.
#[derive(Default)]
pub(crate) struct TypeIdHasher {
    hash: u64
}

impl Hasher for TypeIdHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write_u128(&mut self, i: u128) {
        self.hash = i as u64
    }

    fn write_u64(&mut self, i: u64) {
        self.hash = i
    }

    fn write(&mut self, _bytes: &[u8]) {
        self.hash = 0;

        // Called if TypeId is not a u64 or u128.
        // Fall back to default hasher.
        todo!("Implement TypeIdHasher for non u64 or u128 TypeIds.")
    }
}