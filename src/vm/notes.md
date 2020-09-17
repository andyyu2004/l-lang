Dealing with heap allocation:

Idea:
- disallow explicit borrowing
- i.e. &x will not be an expression
- this is to disallow references to anything stack allocated and memory
  problems
- the only way to obtain a reference/pointer is to use `box expr` where
  the memory is handled by the GC and is memory safe


```
let x = 5;

f(ptr: &int) -> int {
    *int
}

I guess there is no way to call f with x?
Is this reasonable..

I suppose the converse is possible by copying the boxed value into the
stack frame of the function or something?


# TODO
Check for duplicate names in patterns somewhere
