# cwextab

WIP CodeWarrior Extab (Exception Table) decoder tool

## Usage

```rs
use cwextab::{decode_extab, ExceptionTableData};

fn example(extab: &[u8]){
  let result = decode(extab);
  let data = match result {
    Some(val) => val,
    None => {
      panic!(an error happened);
    },
  };
  //do stuffs
}
```
