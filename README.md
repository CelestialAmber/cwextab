# cwextab

WIP CodeWarrior Extab (Exception Table) decoder tool

## Usage

```rs
use cwextab::{decode_extab, ExceptionTableData};

fn example(extab: &[u8]){
  let exData = decode(extab);
  //do stuffs
}
```
