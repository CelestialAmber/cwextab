# cwextab

WIP CodeWarrior Extab (Exception Table) decoder tool

## Usage

```rs
use cwextab::*;

fn example(extab: &[u8]){
  let result = decode_extab(extab);
  let data = match result {
    Some(val) => val,
    None => {
      panic!("An error happened");
    },
  };
  //do stuffs
}
```
