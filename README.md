# typed_shmem
Exposes shared memory on *nix and Windows using mapped files. This work is heavily inspired on the [shared_memory](https://crates.io/crates/shared_memory) crate, but instead of being just a copy cat, **typed_shmem** provides a typed mapping into the shared region.

## Usage
**typed_shmem** is in an early development stage, thus some major changes could be required.
First, a process must create the shared region:
```rust
use typed_shmem as sh;
use typed_shmem::error::ShMemErr;

fn main() -> Result<(), ShMemErr> {
    let mut mem = sh::ShMemCfg::<u32>::default()
         .set_owner()
         .on_file("test_program")
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
use typed_shmem as sh;
use typed_shmem::error::ShMemErr;

fn main() -> Result<(), ShMemErr> {
    let mut mem = sh::ShMemCfg::<u32>::default()
             .on_file("test_program")
             .build()?;
    
    assert_eq!(*mem, 10); //Read.
    
    Ok(())
}
```
## To-Do (no specific order)
- [x] Implement custom error instead of `Box<dyn Error>`ing everything.
- [ ] Implement optional sharing/syncronization mechanisims.
- [ ] Check and rewrite the unsafe blocks (bugs there for sure).
- [ ] Create tests (using `fork()` in *nix (not sure)?; windows?).
- [ ] More to come...

## Contributions
All contributions to this project will be under **Apache-2.0** license.