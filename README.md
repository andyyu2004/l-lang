# L

Rust-inspired language. Very much still a work in progress.

Refer to the [reference](https://l-reference.github.io/reference/)

Development is currently on hold a bit as I work on other things and
think about how to implement memory management (reference counting?) and the trait system.
the trait system.

# Getting Started

Requires LLVM to be installed on your system.

Simply build and run with cargo.


Currently has a (path) dependency on [logic](https://github.com/andyyu2004/logic), and so
`logic` needs to be in the same folder as `l`. Note that `logic` could
be broken at anytime as it is in preliminary stages.

`cargo b --release`
