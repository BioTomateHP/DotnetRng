//! Implementation of .NET's Random algorithm based on Knuth's subtractive method.
//!
//! This crate is extremely lightweight and simple to use.
//! It has zero dependencies and only exports one struct with two methods.
//!
//! It is entirely `no_std` compatible, meaning it can be run in embedded systems without any problems.
//! An allocator is also not needed.
//!
//! All functions are `const`, meaning this crate can be used in const-contexts (at compile time).
//!
//! Reference: <https://github.com/microsoft/referencesource/blob/ec9fa9ae770d522a5b5f0607898044b7478574a3/mscorlib/system/random.cs>

#![no_std]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo, missing_docs)]

/// MSEED is based on an approximation of the golden ratio (phi):
/// MSEED ≈ φ * 10^8
const MSEED: i32 = 161_803_398;

/// A deterministic random number generator.
///
/// This RNG algorithm is used by .NET and is based on Knuth's subtractive method.
///
/// This struct's memory size is platform/architecture independent; it is always 228.
#[derive(Clone)]
pub struct DotnetRng {
    seed_array: [i32; 56],
    inext: u8,
    inextp: u8,
}

impl DotnetRng {
    /// Creates a new [`DotnetRng`] instance based on the given seed.
    #[must_use]
    pub const fn new(seed: i32) -> Self {
        let seed: i32 = if seed == i32::MIN {
            i32::MAX
        } else {
            seed.abs()
        };
        let mut num1: i32 = MSEED.wrapping_sub(seed);
        let mut num2: i32 = 1;
        let mut index1: usize = 0;

        let mut seed_array = [0i32; 56];
        seed_array[55] = num1;

        let mut i: u8 = 1;
        while i < 55 {
            index1 += 21;
            if index1 >= 55 {
                index1 -= 55;
            }
            seed_array[index1] = num2;
            num2 = num1.wrapping_sub(num2);
            if num2 < 0 {
                num2 = num2.wrapping_add(i32::MAX);
            }
            num1 = seed_array[index1];
            i += 1;
        }

        i = 1;
        while i < 5 {
            let mut j: u8 = 1;
            while j < 56 {
                let mut num3: u8 = j + 30;
                if num3 >= 55 {
                    num3 -= 55;
                }

                let seed1: i32 = seed_array[j as usize];
                let seed2: i32 = seed_array[num3 as usize + 1];

                let mut num: i32 = seed1.wrapping_sub(seed2);
                if num < 0 {
                    num = num.wrapping_add(i32::MAX);
                }

                seed_array[j as usize] = num;
                j += 1;
            }
            i += 1;
        }

        Self {
            seed_array,
            inext: 0,
            inextp: 21,
        }
    }

    /// Gets the next pseudorandom 32-bit signed integer based on the state of this [`DotnetRng`] instance.
    ///
    /// For the same internal state, this will always return the same number.
    ///
    /// The state is simply determined by the given seed and how many times [`Self::next`] has been called before.
    pub const fn next(&mut self) -> i32 {
        let mut index1: u8 = self.inext + 1;
        if index1 >= 56 {
            index1 = 1;
        }

        let mut index2: u8 = self.inextp + 1;
        if index2 >= 56 {
            index2 = 1;
        }

        let seed1: i32 = self.seed_array[index1 as usize];
        let seed2: i32 = self.seed_array[index2 as usize];
        let mut num: i32 = seed1.wrapping_sub(seed2);
        if num == i32::MAX {
            num -= 1;
        }
        if num < 0 {
            num = num.wrapping_add(i32::MAX);
        }

        self.seed_array[index1 as usize] = num;
        self.inext = index1;
        self.inextp = index2;

        num
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size() {
        assert_eq!(size_of::<DotnetRng>(), 228);
    }

    #[test]
    fn seed0() {
        let mut rng = DotnetRng::new(0);
        assert_eq!(rng.next(), 1559595546);
        assert_eq!(rng.next(), 1755192844);
        assert_eq!(rng.next(), 1649316166);
        assert_eq!(rng.next(), 1198642031);
        assert_eq!(rng.next(), 442452829);
        assert_eq!(rng.next(), 1200195957);
        assert_eq!(rng.next(), 1945678308);
        assert_eq!(rng.next(), 949569752);
        assert_eq!(rng.next(), 2099272109);
        assert_eq!(rng.next(), 587775847);
    }

    #[test]
    fn seed1000() {
        let mut rng = DotnetRng::new(1000);
        assert_eq!(rng.next(), 325467165);
        assert_eq!(rng.next(), 506683626);
        assert_eq!(rng.next(), 1623525913);
        assert_eq!(rng.next(), 2344573);
        assert_eq!(rng.next(), 1485571032);
        assert_eq!(rng.next(), 980737479);
        assert_eq!(rng.next(), 2067435452);
        assert_eq!(rng.next(), 271829958);
        assert_eq!(rng.next(), 1490890881);
        assert_eq!(rng.next(), 53262104);
    }
}
