# typed_shmem
Exposes shared memory on *nix and Windows using mapped files. This work is heavily inspired on the [shared_memory](https://crates.io/crates/shared_memory) crate, but instead of being just a copy cat, **typed_shmem** provides a typed mapping into the shared memory region.

## Usage
First, a process must create the shared region:
```rust
use typed_shmem as sh;
use typed_shmem::error::ShMemErr;
use typed_shmem::common::ShMemOps;

fn main() -> Result<(), ShMemErr> {
    let mut mem = sh::ShMemCfg::<u32>::default()
        .set_owner()
        .on_file("test_program")
        .build()?;

    //Writing
    unsafe { *mem.get_t_mut() = 10; }

    //Reading
    let val = unsafe { mem.get_t() };
    assert_eq!(*val, 10);

    loop {} //Used to keep the process alive, thus the allocated shared memory too.
     
    Ok(())
```

Then, any other process can join the same region:
```rust
use typed_shmem as sh;
use typed_shmem::error::ShMemErr;
use typed_shmem::common::ShMemOps;

fn main() -> Result<(), ShMemErr> {
    let mut mem = sh::ShMemCfg::<u32>::default()
             .on_file("test_program")
             .build()?;
    
    let val = unsafe { mem.get_t() };
    assert_eq!(*val, 10);
    
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