This tests TLS using dylibs. Spoiler: it will fail when inlining.

Rust dylibs are a strange mix of dll and static library. The upshot of this is that a `#[inline]` function in a dylib can be inlined into another module, whereas `static`s will stay in the dylib. Because Windows TLS are module-local, this will cause the wrong memory location to be accessed when getting or setting the TLS value.
