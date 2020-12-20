#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaskBit {
    False,
    True,
    Neutral,
}

impl From<bool> for MaskBit {
    fn from(b: bool) -> Self {
        use MaskBit::*;
        if b {
            True
        } else {
            False
        }
    }
}

impl From<char> for MaskBit {
    fn from(c: char) -> Self {
        use MaskBit::*;
        match c {
            '0' => False,
            '1' => True,
            _ => Neutral,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Bits {
    bits: [bool; 36],
}
impl std::ops::Deref for Bits {
    type Target = [bool; 36];
    fn deref(&self) -> &Self::Target {
        &self.bits
    }
}
impl std::ops::DerefMut for Bits {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bits
    }
}
impl std::default::Default for Bits {
    fn default() -> Self {
        Self { bits: [false; 36] }
    }
}
impl std::convert::From<usize> for Bits {
    fn from(mut value: usize) -> Self {
        let mut bits_36 = Bits::default();
        for i in (0..36).rev() {
            bits_36[i] = value % 2 == 1;
            value = value / 2;
        }
        bits_36
    }
}
impl std::convert::Into<usize> for Bits {
    fn into(self) -> usize {
        let mut result = 0;
        for &bit in self.into_iter() {
            result *= 2;
            if bit {
                result += 1;
            }
        }
        result
    }
}

#[derive(Debug)]
enum MaskPatch {
    Mask(std::collections::BTreeMap<usize, MaskBit>),
    Mem { offset: usize, value: usize },
}

impl std::str::FromStr for MaskPatch {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use MaskPatch::*;
        if let Some(mask) = s.strip_prefix("mask = ") {
            if mask.len() > 36 {
                panic!("expect the mask to be 36 bits long");
            }
            let bits = mask.chars().map(|c| MaskBit::from(c)).enumerate().collect();
            let mask = Mask(bits);
            return Ok(mask);
        } else if let Some(index) = s.find(']') {
            let offset = s[4..index]
                .parse()
                .expect("expect the offset for 'mem' to be parseable as a 'usize'");
            let value = s[(index + 4)..]
                .parse()
                .expect("expect the value for 'mem' to be parseable as a 'usize'");
            let mem = Mem { offset, value };
            return Ok(mem);
        }
        panic!("expect the line to be either a 'mask' or a 'mem'");
    }
}

#[derive(Debug)]
struct MaskPatches<I>
where
    I: Iterator<Item = &'static str>,
{
    iter: I,
}

impl<I> std::convert::From<I> for MaskPatches<I>
where
    I: Iterator<Item = &'static str>,
{
    fn from(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> std::iter::Iterator for MaskPatches<I>
where
    I: Iterator<Item = &'static str>,
{
    type Item = MaskPatch;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|s| s.parse().expect("expect a valid mask patch"))
    }
}

impl<I> MaskPatches<I>
where
    I: Iterator<Item = &'static str>,
{
    fn apply(self) -> usize {
        use std::collections::BTreeMap;
        let mut memory: BTreeMap<usize, usize> = BTreeMap::new();
        let mut mask = BTreeMap::new();
        for patch in self {
            match patch {
                MaskPatch::Mask(mask_bits) => mask.extend(mask_bits),
                #[cfg(not(feature = "v2"))]
                MaskPatch::Mem { offset, value } => {
                    let mut bits = Bits::from(value);
                    for i in 0..36 {
                        if let Some(mask_bit) = mask.get(&i) {
                            match mask_bit {
                                MaskBit::True => bits[i] = true,
                                MaskBit::False => bits[i] = false,
                                MaskBit::Neutral => (),
                            }
                        }
                    }
                    memory.insert(offset, bits.into());
                }
                #[cfg(feature = "v2")]
                MaskPatch::Mem { offset, value } => {
                    let mut addresses = std::collections::HashSet::new();
                    addresses.insert(Bits::from(offset));
                    for i in 0..36 {
                        addresses = {
                            let mut new_addresses = std::collections::HashSet::new();
                            for mut address in addresses.into_iter() {
                                match mask[&i] {
                                    MaskBit::False => {
                                        new_addresses.insert(address);
                                    }
                                    MaskBit::True => {
                                        address[i] = true;
                                        new_addresses.insert(address);
                                    }
                                    MaskBit::Neutral => {
                                        let mut toggle_address = address.clone();
                                        address[i] = true;
                                        new_addresses.insert(address);
                                        toggle_address[i] = false;
                                        new_addresses.insert(toggle_address);
                                    }
                                }
                            }
                            new_addresses
                        };
                    }
                    for address in addresses {
                        memory.insert(address.into(), value);
                    }
                }
            }
        }
        memory.values().sum()
    }
}

fn main() {
    let sum = MaskPatches::from(include_str!("../masks.txt").trim().split('\n')).apply();
    println!("Sum of all in-memory values is {}", sum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usize_to_bits() {
        let bits = Bits::from(6);
        assert_eq!(false, bits[35]);
        assert_eq!(true, bits[34]);
        assert_eq!(true, bits[33]);
        for i in 0..32 {
            assert_eq!(false, bits[i]);
        }
        let value: usize = bits.into();
        assert_eq!(6, value);
        let bits = Bits::from(3);
        let value: usize = bits.into();
        assert_eq!(3, value);
    }

    #[cfg(not(feature = "v2"))]
    #[test]
    fn docking_data() {
        let masks = r#"mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
mem[8] = 11
mem[7] = 101
mem[8] = 0"#;
        let sum = MaskPatches::from(masks.split('\n')).apply();
        assert_eq!(165, sum);
    }

    #[cfg(feature = "v2")]
    #[test]
    fn docking_data() {
        let masks = r#"mask = 000000000000000000000000000000X1001X
mem[42] = 100
mask = 00000000000000000000000000000000X0XX
mem[26] = 1"#;
        let sum = MaskPatches::from(masks.split('\n')).apply();
        assert_eq!(208, sum);
    }
}
