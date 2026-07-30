## Libmuslim-rs

Libmuslim-rs is Rust binding for [libmuslim](https://github.com/muslimtify-org/libmuslim), a collection of muslim useful tools and library written in C wrapping into safe Rust idiomatic code convention, this library covers:

1. prayertimes.h: an Astronomical calculations for prayer times, support 21 international standard calculation
2. hijri.h: A from-scratch astronomical Hijri (Islamic lunar) calendar library (not yet)

for more information, about usage and API documentation, please visit [muslimtify](https://muslimtify.vercel.app) documentation website

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
libmuslim-rs = "0.1.0"
```

or run this command in your terminal:

```bash
cargo add libmuslim-rs
```

## Usage

please refer to this [example](examples/basic.rs)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
