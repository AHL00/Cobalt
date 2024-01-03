use hashbrown::HashMap;

/// Data structure that holds data in a contiguous array and allows for O(1) insertion and removal.
/// When storing new data, it will be placed in the first available slot and will be stored with the given key.
/// When retrieving data, the key will be used to find the data.
pub struct SlotMap<K, V>
where K: Eq + std::hash::Hash
{
    data: Vec<V>,
    free_slots: Vec<usize>,
    map: HashMap<K, usize>,
}

impl<K, V> SlotMap<K, V>
where K: Eq + std::hash::Hash
{
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            free_slots: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            free_slots: Vec::new(),
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn add(&mut self, key: K, value: V) -> Result<(), Box<dyn std::error::Error>> {
        let index = if let Some(index) = self.free_slots.pop() {
            index
        } else {
            self.data.push(value);

            self.data.len() - 1
        };
        
        self.map.insert(key, index);

        Ok(())
    }

    pub fn remove(&mut self, key: &K) -> Result<(), Box<dyn std::error::Error>> {
        let index = self.map.remove(key).ok_or("Key does not exist.")?;

        self.data.remove(index);

        Ok(())
    }

    pub fn get_mut(&mut self, key: &K) -> Result<&mut V, Box<dyn std::error::Error>> {
        let index = self.map.get(key).ok_or("Key does not exist.")?;

        Ok(self.data.get_mut(*index).unwrap())
    }

    pub fn get(&self, key: &K) -> Result<&V, Box<dyn std::error::Error>> {
        let index = self.map.get(key).ok_or("Key does not exist.")?;

        Ok(self.data.get(*index).unwrap())
    }
}

struct SlotMapIter<'a, K, V>
where K: Eq + std::hash::Hash
{
    slot_map: &'a SlotMap<K, V>,
    iter: hashbrown::hash_map::Iter<'a, K, usize>,
}

impl<K, V> SlotMap<K, V>
where K: Eq + std::hash::Hash
{
    pub fn iter(&self) -> SlotMapIter<K, V> {
        SlotMapIter {
            slot_map: self,
            iter: self.map.iter(),
        }
    }
}

impl<'a, K, V> Iterator for SlotMapIter<'a, K, V>
where K: Eq + std::hash::Hash
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, index) = self.iter.next()?;

        Some((key, self.slot_map.data.get(*index).unwrap()))
    }
}