# DotnetRNG

An implementation of .NET's Random algorithm based on Knuth's subtractive method.

This crate is extremely lightweight and simple to use.
It has zero dependencies and only exports one struct with two methods.

It is entirely `no_std` compatible, meaning it can be run in embedded systems without any problems.
An allocator is also not needed.

All functions are `const`, meaning this crate can be used in const-contexts (at compile time).

Reference: <https://github.com/microsoft/referencesource/blob/ec9fa9ae770d522a5b5f0607898044b7478574a3/mscorlib/system/random.cs> (accessed: 2026-03-23)

## License
This crate is re-licensed under the [MIT license](https://opensource.org/license/mit).
See the attached [LICENSE.md](LICENSE.md) file for more information.
