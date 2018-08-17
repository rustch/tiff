# TIFF

Access TIFF image using Rust

# Reading

```rust
let bytes: &[u8] = include_bytes!("../samples/ycbcr-cat.tif");
let mut cursor = Cursor::new(bytes);
let mut read = TIFF::new(&mut cursor).unwrap();
let field = read.get_field::<YResolution>().unwrap();
print("YResolution: {}", field.0);
```