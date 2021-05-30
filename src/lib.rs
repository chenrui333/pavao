//! # Pavão
//!
//! [Pavão](https://github.com/veeso/pavao) is a Rust native async client library for SMB2/SMB3
//!
//! ## Get Started
//!
//! ### Adding `pavao` to your cargo toml dependencies:
//!
//! ```toml
//! pavao = "0.1.0"
//! ```
//!

#![doc(html_playground_url = "https://play.rust-lang.org")]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/veeso/pavao/main/docs/images/cargo/pavao-128.png"
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/veeso/pavao/main/docs/images/cargo/pavao-512.png"
)]

/**
 * MIT License
 *
 * pavao - Copyright (C) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
// deps
extern crate async_std;
#[macro_use]
extern crate bitflags;
extern crate bytes;
#[macro_use]
extern crate log;
extern crate num;
#[macro_use]
extern crate num_derive;
extern crate rand;
// public
pub mod smb2;
// private global
pub(crate) mod socket;
mod utils;
