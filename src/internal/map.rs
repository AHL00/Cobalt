

// /// An map type. Stores key-value pairs in a vector.
// /// The key and value types must be the same.
// /// Useful for storing indexes in data structures to their IDs.
// pub struct Map<T>
// {
//     /// Data is stored as follows:
//     /// [key[0], value[0], key[1], value[1], ...]
//     data: Vec<T>,
// }

// impl<T> Map<T> {
//     /// Creates a new map with the given initial capacity.
//     pub fn with_capacity(capacity: usize) -> Self
//     {
//         Self
//         {
//             data: Vec::with_capacity(capacity * 2),
//         }
//     }
    
//     pub fn new() -> Self
//     {
//         Self
//         {
//             data: Vec::new(),
//         }
//     }

//     /// Inserts a new key-value pair into the map.
//     /// If the key already exists, the value is updated.
//     pub fn insert(&mut self, key: usize, value: T)
//     {
//         // Check if the key already exists.
//         if let Some(index) = self.find_index(key)
//         {
//             // Update the value.
//             self.data[index + 1] = value;
//         }
//         else
//         {
//             // Insert the key-value pair.
//             self.data.push(key);
//             self.data.push(value);
//             self.len += 1;
//         }
//     }

//     /// Removes the key-value pair with the given key from the map.
//     pub fn remove(&mut self, key: usize)
//     {
//         // Check if the key exists.
//         if let Some(index) = self.find_index(key)
//         {
//             // Remove the key-value pair.
//             self.data.remove(index);
//             self.data.remove(index);
//             self.len -= 1;
//         }
//     }

//     /// Returns the value associated with the given key.
//     pub fn get(&self, key: usize) -> Option<&T>
//     {
//         // Check if the key exists.
//         if let Some(index) = self.find_index(key)
//         {
//             // Return the value.
//             Some(&self.data[index + 1])
//         }
//         else
//         {
//             None
//         }
//     }

//     /// Returns the value associated with the given key.
//     pub fn get_mut(&mut self, key: usize) -> Option<&mut T>
//     {
//         // Check if the key exists.
//         if let Some(index) = self.find_index(key)
//         {
//             // Return the value.
//             Some(&mut self.data[index + 1])
//         }
//         else
//         {
//             None
//         }
//     }

// }