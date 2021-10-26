Allows creating static thread locals on Windows. This requires a nightly compiler
and `#![feature(asm)]`.

See the [documentation](https://chrisdenton.github.io/wintls/wintls/index.html)

# Example

```rust
#![feature(asm)]

wintls::static_thread_local!{
    static TEST: u32 = 0;
}

fn main() {
    let value = TEST.get();
    TEST.set(value + 1);
}
```