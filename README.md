# twigmd
twigmd is a small markdown parser that generates simple tree structures from markdown strings.

## How to use
```rust
let input = "- Item 1¥\n - Subitem 1.1\n- Item 2";
let tree = build_tree(input);

for node in tree {
    println!("{:?}", node);
}
```

## Author
Masahiro Shimizu (@okonomipizza)

## License
MIT