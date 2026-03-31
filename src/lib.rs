//! An implementation of .NET's Random algorithm based on Knuth's subtractive method.
//!
//! This crate is:
//! * extremely lightweight (no dependencies, low amount of code)
//! * compatible with embedded systems (`no_std` and `no_alloc`)
//! * entirely usable in constant evaluation (all functions are marked as `const`)
//! * platform-independent (no usage of `usize` or pointers in `struct`s)
//!
//! # Usage
//! ```
//! use dotnet_rng::DotnetRng;
//!
//! // Create a new RNG instance with a given seed
//! let mut rng = DotnetRng::new(1337);
//!
//! // Generate integer between [-2147483648, 2147483648)
//! let num: i32 = rng.next();
//!
//! // Generate integer between [100, 200)
//! let num: i32 = rng.next_ranged(100, 200);
//!
//! // Advance internal state (same as discarding rng.next() return value)
//! rng.skip();
//!
//! // Generate number between [0, 1)
//! let num: f64 = rng.next_f64();
//!
//! // Generate 64 random bytes
//! let bytes: [u8; 64] = rng.next_bytes();
//!
//! // Fill existing byte buffer
//! let mut buffer = [0u8; 100];
//! rng.fill_bytes(&mut buffer);
//! println!("Bytes: {buffer:?}");
//!
//! // RNG is deterministic
//! let mut new_rng = rng.clone();
//! assert_eq!(rng.next(), new_rng.next());
//! assert_eq!(rng.next_f64(), new_rng.next_f64());
//! ```
//!
//! # Reference
//! The algorithm is taken from
//! <https://github.com/microsoft/referencesource/blob/ec9fa9ae770d522a5b5f0607898044b7478574a3/mscorlib/system/random.cs>
//! (accessed: 2026-03-31).

#![no_std]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo, missing_docs)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

/// MSEED is based on an approximation of the golden ratio (phi):
/// MSEED ≈ φ * 10^8
const MSEED: i32 = 161_803_398;

/// A deterministic random number generator.
///
/// This RNG algorithm is used by .NET and is based on Knuth's subtractive method.
///
/// For more information, see the [module level documentation](self).
#[derive(Clone)]
pub struct DotnetRng {
    seed_array: [i32; 56],
    inext: u8,
    inextp: u8,
}

impl DotnetRng {
    /// Creates a new [`DotnetRng`] instance based on the given seed.
    ///
    /// All methods will return the same "random" numbers for an RNG
    /// instance with the same seed and state. The state is determined
    /// by how many times [`DotnetRng::next`] has been called in total.
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

    /// Generates a new number and throws it away.
    ///
    /// This is useful if you want to advance the RNG's internal
    /// state without actually using the generated number.
    #[inline]
    pub const fn skip(&mut self) {
        let _ = self.next();
    }

    /// Gets the next pseudorandom 32-bit signed integer based on the state of this [`DotnetRng`] instance.
    ///
    /// **Range**: [`i32::MIN`] .. [`i32::MAX`]
    ///
    /// For the same internal state, this will always return the same number.
    ///
    /// The state is simply determined by the given seed and how many times [`Self::next`] has been called before.
    #[must_use = "if you intend to only advance the internal rng state, use `.skip()`"]
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

    /// Gets the next signed 32-bit integer within the given range.
    ///
    /// **Range**: `min` .. `max`
    ///
    /// If the range is [`i32::MAX`] or larger, a slighty different algorithm
    /// is used which internally calls `.next()` twice instead of once.
    ///
    /// For more information on number generation, see [`DotnetRng::next`].
    ///
    /// # Panics
    /// This function will panic if `min > max`.
    #[must_use = "if you intend to only advance the internal rng state, use `.skip()`"]
    pub const fn next_ranged(&mut self, min: i32, max: i32) -> i32 {
        assert!(min <= max, "minimum is greater than maximum");

        if let Some(range) = max.checked_sub(min) {
            return (self.next_f64() * (range as f64)) as i32 + min;
        }

        // Large range; more steps needed.
        let mut sample: i32 = self.next();
        if self.next() % 2 == 0 {
            sample = -sample;
        }
        let mut num: f64 = sample as f64;
        num += (i32::MAX - 1) as f64;
        num /= (2 * (i32::MAX) as u32) as f64 - 1.0;

        let range: f64 = (max as f64) - (min as f64);
        (num * range) as i32 + min
    }

    /// Gets the next double-precision floating point number.
    ///
    /// **Range**: 0 .. 1
    ///
    /// For more information on number generation, see [`DotnetRng::next`].
    #[doc(alias = "next_double")]
    #[inline]
    #[must_use = "if you intend to only advance the internal rng state, use `.skip()`"]
    pub const fn next_f64(&mut self) -> f64 {
        self.next() as f64 * (1.0 / i32::MAX as f64)
    }

    /// Fills a given buffer with random bytes.
    ///
    /// For each byte, `.next()` is called and its return value is truncated to an unsigned 8-bit integer.
    /// The internal state is therefore advanced `buffer.len()` times.
    ///
    /// If you have a known array size at compile-time, consider using
    /// [`DotnetRng::next_bytes`] instead.
    ///
    /// For more information on number generation, see [`DotnetRng::next`].
    pub const fn fill_bytes(&mut self, buffer: &mut [u8]) {
        let mut i = 0;
        while i < buffer.len() {
            buffer[i] = self.next() as u8;
            i += 1;
        }
    }

    /// Creates and fills a buffer with random bytes.
    ///
    /// For each byte, `.next()` is called and its return value is truncated to an unsigned 8-bit integer.
    /// The internal state is therefore advanced `N` times.
    ///
    /// If you do not have a known array size at compile-time, consider using [`DotnetRng::fill_bytes`].
    ///
    /// For more information on number generation, see [`DotnetRng::next`].
    #[must_use = "if you intend to only advance the internal rng state, use `.skip()`"]
    pub const fn next_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut buffer = [0u8; N];
        self.fill_bytes(&mut buffer);
        buffer
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
    fn bytes() {
        let mut rng = DotnetRng::new(-1337);
        let bytes: [u8; 100] = rng.next_bytes();
        assert_eq!(
            bytes,
            [
                104, 0, 244, 199, 67, 94, 2, 170, 194, 124, 79, 217, 39, 252, 34, 39, 106, 137, 84,
                178, 229, 18, 239, 30, 154, 247, 34, 126, 240, 54, 227, 40, 165, 116, 212, 193, 23,
                186, 227, 105, 199, 86, 230, 13, 79, 164, 218, 69, 90, 187, 243, 186, 246, 89, 36,
                85, 16, 214, 45, 76, 60, 132, 185, 139, 152, 38, 51, 179, 39, 97, 15, 176, 166,
                235, 234, 143, 44, 226, 206, 246, 29, 221, 35, 52, 67, 41, 50, 76, 79, 127, 177,
                65, 141, 150, 44, 67, 156, 90, 117, 41
            ]
        );
    }

    #[test]
    fn seed0() {
        let mut rng = DotnetRng::new(0);
        assert_eq!(rng.next(), 1_559_595_546);
        assert_eq!(rng.next(), 1_755_192_844);
        assert_eq!(rng.next(), 1_649_316_166);
        assert_eq!(rng.next(), 1_198_642_031);
        assert_eq!(rng.next(), 442_452_829);
        assert_eq!(rng.next(), 1_200_195_957);
        assert_eq!(rng.next(), 1_945_678_308);
        assert_eq!(rng.next(), 949_569_752);
        assert_eq!(rng.next(), 2_099_272_109);
        assert_eq!(rng.next(), 587_775_847);
    }

    #[test]
    fn seed1000() {
        let mut rng = DotnetRng::new(1000);
        assert_eq!(rng.next(), 325_467_165);
        assert_eq!(rng.next(), 506_683_626);
        assert_eq!(rng.next(), 1_623_525_913);
        assert_eq!(rng.next(), 2_344_573);
        assert_eq!(rng.next(), 1_485_571_032);
        assert_eq!(rng.next(), 980_737_479);
        assert_eq!(rng.next(), 2_067_435_452);
        assert_eq!(rng.next(), 271_829_958);
        assert_eq!(rng.next(), 1_490_890_881);
        assert_eq!(rng.next(), 53_262_104);
    }

    #[test]
    fn doubles() {
        fn assert_eq(generated: f64, known: f64) {
            let epsilon: f64 = 0.000_000_000_000_001;
            let diff = (generated - known).abs();
            assert!(diff < epsilon);
        }

        let mut rng = DotnetRng::new(1225);
        assert_eq(rng.next_f64(), 0.697_253_151_655_781);
        assert_eq(rng.next_f64(), 0.255_131_907_414_241);
        assert_eq(rng.next_f64(), 0.028_311_024_433_146_7);
        assert_eq(rng.next_f64(), 0.025_751_158_607_076_5);
        assert_eq(rng.next_f64(), 0.276_064_366_696_432);
        assert_eq(rng.next_f64(), 0.083_697_980_774_425_9);
        assert_eq(rng.next_f64(), 0.775_481_611_851_361);
        assert_eq(rng.next_f64(), 0.005_571_321_586_878_65);
        assert_eq(rng.next_f64(), 0.530_507_760_369_455);
        assert_eq(rng.next_f64(), 0.993_799_069_427_792);
        assert_eq(rng.next_f64(), 0.786_803_013_080_174);
        assert_eq(rng.next_f64(), 0.842_510_045_432_723);
    }

    #[test]
    fn ranged_large() {
        let mut rng = DotnetRng::new(20_230_807);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 1_534_921_242);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 2_038_703_413);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 998_041_784);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 1_589_122_846);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 1_789_172_735);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 629_506_782);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 723_391_659);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 1_828_598_720);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 1_160_835_804);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 1_544_888_066);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 1_039_720_213);
        assert_eq!(rng.next_ranged(-42069, i32::MAX), 280_809_207);
    }
}
