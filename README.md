# Simple undo

An easy to use undo-redo library:

```rust
use simple_undo::Undo;

let mut message = Undo::new(String::new());
message.update(|text| text.push_str("Simple "));
message.update(|text| text.push_str("undo !"));
assert_eq!(*message, "Simple undo !");

message.undo(); // "Simple "
message.undo(); // ""
message.redo(); // "Simple "

message.update(|text| text.push_str("redo !"));
assert_eq!(*message, "Simple redo !");

let result: String = message.unwrap();
assert_eq!(result, "Simple redo !");
```

## How it works

`Undo` wraps the given state and keeps one copy of it.
When [`Undo::undo`] is called, the previous state is re-created by re-applying the n-1 updates to the initial state.

If you need better performance, please consider alternatives such as [`undo`](https://lib.rs/crates/undo) or [`rundo`](https://lib.rs/crates/rundo) crates, which allow you to define or generate the actual undo operation.
