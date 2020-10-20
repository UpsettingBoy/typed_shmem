# typed_shmem
Exposes shared memory on *nix and Windows using mapped files. This work is heavily inspired on the [https://crates.io/crates/shared_memory](shared_memory) crate, but instead of being just a copy cat, **typed_shmem** provides a typed mapping into the shared region.

## Usage
**typed_shmem** is in an early development stage, thus some major changes could be required, expect some API stability.

First, a process must create the shared region:
```rust
use std::error::Error;
use typed_shmem as sh;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = sh::ShMemCfg::<u32>::default()
         .owner()
         .on_file("test_program".to_string())
         .build()?;
    
    // ShMem<T> implements Deref and DerefMut.
    *mem = 10; //Write.
    assert_eq!(*mem, 10); //Read.
    
    loop {} //Used to keep the process alive, thus the allocated shared memory too.
    
    Ok(())
}
```

Then, any other process can join the same region:
```rust
use std::error::Error;
use typed_shmem as sh;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = sh::ShMemCfg::<u32>::default()
             .on_file("test_program".to_string())
             .build()?;
    
    assert_eq!(*mem, 10); //Read.
    
    Ok(())
}
```
## To-Do (no specific order)
- [] Implement custom error instead of `Box<dyn Error>`ing everything.
- [] Reduce memory allocations for the creation of `ShMem` throught `ShMemCfg` (`String` related), although is not a major bottleneck/issue.
- [] Implement optional sharing/syncronization mechanisims.
- [] Check and rewrite the unsafe blocks (bugs there for sure).
- [] Move invalid target into compile time instead of `panic!`ing at runtime.
- [] Create tests (using `fork()` in *nix (not sure)?; windows?).
- [] More to come...

## Contributions
All contributions to this project will be under **Apache-2.0** license.