# TODO

## Bugs

### Function Calls messes up stack index of variables

In a simple program like 

```lisp
(defun main {
  (defvar $i 5)
  (while (< $i 7) {
    (print $i)
    (setvar $i (+ $i 1))
  })
  (exit)
})
```

The `(print $i)` statement pushes `0` onto the stack and is never popped
which means that when `$i` is referenced at `s(0)` in the next iterations 
comparison it will no longer be `$i` (now at `s(1)`) since `s(0)` now points to the `0` that 
the `(print $i)` statement pushed.

This is fine for the first iteration as the push value from `(print $i)` is
accounted for in the while-body, but when it jumps up again to perform the
comparison the `$i` var has moved up one value so the comparison between `s(0)`
and `7` will not be between `$i` and `7` but instead between the `0` that the 
`(print $i)` statement returned and `7`. This will repeat, making it stuck in
an infinite loop, adding one more `0` to the stack each time.
