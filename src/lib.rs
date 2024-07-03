//! A password generator that can be used as a library or at the command line
//!
//! When using from the command line can either provide a spec string, or override the default (or
//! current spec string) with other arguments.
//!
//!
//! # CLI examples
//! Using the default spec
//! ```bash
//! $ pants-gen
//! PHk};IUX{59!H88252x4wjD(Fg|5cva|
//! ```
//!
//! Overriding the default spec to be:
//!  - 3 or more uppercase letters
//!  - 1 to 2 lowercase letters
//!  - 3 or fewer numbers
//!  - 1 symbol
//!  - password of length 16
//! ```bash
//! $ pants-gen --spec '16//3+|:upper://1-2|:lower://3-|:number://1|:symbol:'
//! 8Z6TWWCARwJxC)8C
//! ```
//!
//! Overriding parts of the default spec
//!  - setting the length to be 12
//! ```bash
//! $ pants-gen -l 12
//! bS),2VMV2G+T
//! ```
//!
//! Setting custom charater groups
//!  - disabling the symbols
//!  - setting an equivalent set of symbols to be !@#$%^&*|_+-=
//! ```bash
//! $ pants-gen -s 0 -c '!@#$%^&*|_+-=|1+'
//! =LsI8=%@%GP5hMlIm%#dj9&66V9-#7h@
//! ```
//!
//! # Library examples
//!
//! To generate a password build up the spec and then call `generate` to produce the password. This
//! function returns an `Option` since the constraints on the provided choices can't always meet
//! the length requirement given.
//! ```rust
//! use pants_gen::password::{PasswordSpec, CharStyle};
//! use pants_gen::interval::Interval;
//! let spec = PasswordSpec::new()
//!     .length(16)
//!     .upper_at_least(1)
//!     .lower(Interval::new(1,10).unwrap())
//!     .include(CharStyle::Number.exactly(3))
//!     .custom(vec!['&', '^'], Interval::exactly(1));
//! if let Some(p) = spec.generate() {
//!     println!("{}", p);
//! } else {
//!     println!("Couldn't meet constraints of spec");
//! }
//! ```
pub mod cli;
pub mod interval;
pub mod password;
