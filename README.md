# cwextab [![Latest Version]][crates.io] [![Api Rustdoc]][rustdoc] ![Rust Version]

[Latest Version]: https://img.shields.io/crates/v/cwextab.svg
[crates.io]: https://crates.io/crates/cwextab
[Api Rustdoc]: https://img.shields.io/badge/api-rustdoc-blue.svg
[rustdoc]: https://docs.rs/cwextab
[Rust Version]: https://img.shields.io/badge/rust-1.58+-blue.svg?maxAge=3600


WIP CodeWarrior Extab (Exception Table) decoder tool

## Usage

```rs
use cwextab::*;

fn example(extab: &[u8]){
  let result = decode_extab(extab);
  let data = match result {
    Ok(val) => val,
    Err(e) => {
      panic!("An error happened: {}", e);
    },
  };
  //do stuffs
}
```
