// TODO: SIMD rewrite for bitwise operations and comparisons

/// Bit array that is stored in a fixed-size array.
/// Array size granularity is 8 bits.
#[derive(Clone, Debug)]
pub struct BitArray<const N: usize>
where
    [(); N / 8]:,
{
    data: [u8; N / 8],
}

impl<const N: usize> BitArray<N>
where
    [(); N / 8]:,
{
    pub fn new() -> Self {
        Self { data: [0; N / 8] }
    }

    pub fn set(&mut self, index: usize, value: bool) {
        let byte_index = index / 8;
        let bit_index = index % 8;

        if value {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }

    pub fn get(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = index % 8;

        (self.data[byte_index] & (1 << bit_index)) != 0
    }
    
    /// Returns true if the bit array contains all of the bits in the other bit array.
    pub fn contains(&self, other: &Self) -> bool {
        for i in 0..N / 8 {
            if (self.data[i] & other.data[i]) != other.data[i] {
                return false;
            }
        }

        true
    }

    pub fn iter(&self) -> BitArrayIter<N> {
        BitArrayIter {
            bit_array: self,
            index: 0,
        }
    }
}

pub struct BitArrayIter<'a, const N: usize>
where
    [(); N / 8]:,
{
    bit_array: &'a BitArray<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for BitArrayIter<'a, N>
where
    [(); N / 8]:,
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.bit_array.get(self.index);

        self.index += 1;

        Some(value)
    }
}

impl<const N: usize> std::ops::BitAnd for BitArray<N>
where
    [(); N / 8]:,
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut result = Self::new();

        for i in 0..N / 8 {
            result.data[i] = self.data[i] & rhs.data[i];
        }

        result
    }
}

impl<const N: usize> std::ops::BitOr for BitArray<N>
where
    [(); N / 8]:,
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut result = Self::new();

        for i in 0..N / 8 {
            result.data[i] = self.data[i] | rhs.data[i];
        }

        result
    }
}

impl<const N: usize> std::ops::BitXor for BitArray<N>
where
    [(); N / 8]:,
{
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut result = Self::new();

        for i in 0..N / 8 {
            result.data[i] = self.data[i] ^ rhs.data[i];
        }

        result
    }
}

impl<const N: usize> std::ops::Not for BitArray<N>
where
    [(); N / 8]:,
{
    type Output = Self;

    fn not(self) -> Self::Output {
        let mut result = Self::new();

        for i in 0..N / 8 {
            result.data[i] = !self.data[i];
        }

        result
    }
}



impl<const N: usize> PartialEq for BitArray<N>
where
    [(); N / 8]:,
{
    fn eq(&self, other: &Self) -> bool {
        for i in 0..N / 8 {
            if self.data[i] != other.data[i] {
                return false;
            }
        }

        true
    }
}