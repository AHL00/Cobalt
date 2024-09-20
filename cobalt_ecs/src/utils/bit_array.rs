#![allow(dead_code)]

use std::simd::Simd;

use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};

/// Stack allocated bit array that uses SIMD instructions for bitwise operations and comparisons.
/// The array size is a multiple of the maximum SIMD instruction set size. Uses 256-bit SIMD vectors,
/// some CPUs may lack support it, but it should be more common in the future.
/// `N` is the number of bits in the bit array. It must be greater than 255 and ideally a multiple of 256.
#[derive(Debug, Clone)]
pub struct SimdBitArray<const N: usize>
where
    [(); N / 256]:,
{
    data: [Simd<u8, 32>; N / 256],
}

impl<const N: usize> SimdBitArray<N>
where
    [(); N / 256]:,
{
    pub fn new() -> Self {
        Self {
            data: [Simd::splat(0); N / 256],
        }
    }

    pub fn fill(&mut self, value: bool) {
        for i in 0..N / 256 {
            if value {
                self.data[i] = Simd::splat(u8::MAX);
            } else {
                self.data[i] = Simd::splat(0);
            }
        }
    }

    /// Sets a single bit at the given index.
    pub fn set(&mut self, index: usize, value: bool) {
        let vector_index = index / 512;
        let bit_index = index % 512;
        let lane_index = bit_index / 8;
        let byte = 1u8 << (bit_index % 8);

        if value {
            let mut temp = self.data[vector_index];
            temp[lane_index] |= byte;
            self.data[vector_index] = temp;
        } else {
            let mut temp = self.data[vector_index];
            temp[lane_index] &= !byte;
            self.data[vector_index] = temp;
        }
    }

    pub fn get(&self, index: usize) -> bool {
        let vector_index = index / 512;
        let lane_index = (index % 512) / 8;
        let bit_index = index % 8;

        // Get vector, and then extract the lane.
        let vector = &self.data[vector_index];
        let byte = vector[lane_index];

        let mask = 1 << bit_index;

        (byte & mask) != 0
    }

    /// Returns whether this bit array is a superset of the other bit array.
    pub fn contains(&self, other: &Self) -> bool {
        for i in 0..N / 256 {
            if (self.data[i] & other.data[i]) != other.data[i] {
                return false;
            }
        }

        true
    }
}

impl<const N: usize> PartialEq for SimdBitArray<N>
where
    [(); N / 256]:,
{
    fn eq(&self, other: &Self) -> bool {
        for i in 0..N / 256 {
            if self.data[i] != other.data[i] {
                return false;
            }
        }

        true
    }
}

impl<const N: usize> Serialize for SimdBitArray<N>
where
    [(); N / 256]:,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(N / 256))?;

        // Serialize as a sequence of 128-bit integers.
        for i in 0..N / 256 {
            let mut val1 = 0u128;
            for j in 0..16 {
                val1 |= (self.data[i][j] as u128) << (j * 8);
            }

            let mut val2 = 0u128;
            for j in 16..32 {
                val2 |= (self.data[i][j] as u128) << ((j - 16) * 8);
            }

            seq.serialize_element(&val1)?;
            seq.serialize_element(&val2)?;
        }

        seq.end()
    }
}

impl<'a, const N: usize> Deserialize<'a> for SimdBitArray<N>
where
    [(); N / 256]:,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        struct SimdBitArrayVisitor<const N: usize>
        where
            [(); N / 256]:,
        {
            _marker: std::marker::PhantomData<SimdBitArray<N>>,
        }

        impl<'a, const N: usize> serde::de::Visitor<'a> for SimdBitArrayVisitor<N>
        where
            [(); N / 256]:,
        {
            type Value = SimdBitArray<N>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of 128-bit integers")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'a>,
            {
                let mut bit_array = SimdBitArray::<N>::new();

                for i in 0..N / 256 {
                    let val1: u128 = seq.next_element()?.unwrap();
                    let val2: u128 = seq.next_element()?.unwrap();

                    // Fill the first 128 bits.
                    for j in 0..16 {
                        let mut temp = bit_array.data[i];
                        temp[j] = (val1 >> (j * 8)) as u8;
                        bit_array.data[i] = temp;
                    }

                    // Fill the second 128 bits.
                    for j in 16..32 {
                        let mut temp = bit_array.data[i];
                        temp[j] = (val2 >> ((j - 16) * 8)) as u8;
                        bit_array.data[i] = temp;
                    }
                }

                Ok(bit_array)
            }
        }

        deserializer.deserialize_seq(SimdBitArrayVisitor::<N> {
            _marker: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simd_bit_array_set_get() {
        let mut bit_array = SimdBitArray::<1024>::new();

        bit_array.set(0, true);
        bit_array.set(8, false);

        assert_eq!(bit_array.get(0), true);
        assert_eq!(bit_array.get(8), false);

        bit_array.set(0, false);
        bit_array.set(8, true);

        assert_eq!(bit_array.get(0), false);
        assert_eq!(bit_array.get(8), true);
    }

    #[test]
    fn simd_bit_array_contains() {
        let mut bit_array1 = SimdBitArray::<1024>::new();
        let mut bit_array2 = SimdBitArray::<1024>::new();

        bit_array1.set(0, true);
        bit_array1.set(8, false);

        bit_array2.set(0, true);

        assert!(bit_array1.contains(&bit_array2));

        bit_array2.set(8, true);

        assert!(!bit_array1.contains(&bit_array2));
    }

    #[test]
    fn simd_bit_array_contains_empty() {
        let mut bit_array1 = SimdBitArray::<1024>::new();
        let bit_array2 = SimdBitArray::<1024>::new();

        bit_array1.set(0, true);
        bit_array1.set(8, false);

        assert!(bit_array1.contains(&bit_array2));
    }

    #[test]
    fn simd_bit_array_partial_eq() {
        let mut bit_array1 = SimdBitArray::<1024>::new();
        let mut bit_array2 = SimdBitArray::<1024>::new();

        bit_array1.set(0, true);
        bit_array1.set(8, false);

        bit_array2.set(0, true);
        bit_array2.set(8, false);

        assert_eq!(bit_array1, bit_array2);

        bit_array2.set(0, false);

        assert_ne!(bit_array1, bit_array2);
    }

    #[test]
    fn serde_simd_bit_array() {
        let mut bit_array = SimdBitArray::<256>::new();

        for i in 0..256 {
            bit_array.set(i, i % 3 == 0);
        }

        let serialized = serde_yaml::to_string(&bit_array).unwrap();

        let deserialized: SimdBitArray<256> = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(bit_array, deserialized);
    }
}
