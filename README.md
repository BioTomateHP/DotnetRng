# DotnetRNG

A Rust implementation of .NET's Random algorithm based on Knuth's subtractive method.

This crate is:
* extremely lightweight (no dependencies, low amount of code)
* compatible with embedded systems (`no_std` and `no_alloc`)
* entirely usable in constant evaluation (all functions are marked as `const`)
* platform-independent (no usage of `usize` or pointers in `struct`s)

## Usage
```rust
use dotnet_rng::DotnetRng;

// Create a new RNG instance with a given seed
let mut rng = DotnetRng::new(1337);

// Generate integer between [-2147483648, 2147483648)
let num: i32 = rng.next();

// Generate integer between [100, 200)
let num: i32 = rng.next_ranged(100, 200);

// Advance internal state (same as discarding rng.next() return value)
rng.skip();

// Generate number between [0, 1)
let num: f64 = rng.next_f64();

// Generate 64 random bytes
let bytes: [u8; 64] = rng.next_bytes();

// Fill existing byte buffer
let mut buffer = [0u8; 100];
rng.fill_bytes(&mut buffer);
println!("Bytes: {buffer:?}");

// RNG is deterministic
let mut new_rng = rng.clone();
assert_eq!(rng.next(), new_rng.next());
assert_eq!(rng.next_f64(), new_rng.next_f64());
```

## Reference 
The algorithm is taken from
<https://github.com/microsoft/referencesource/blob/ec9fa9ae770d522a5b5f0607898044b7478574a3/mscorlib/system/random.cs>
(accessed: 2026-03-31).

## License
The original algorithm was made by Microsoft.
This Rust port was made by BioTomateDE.

This crate is re-licensed under the [MIT license](https://opensource.org/license/mit).
See the attached [LICENSE] file for more information.
