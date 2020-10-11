# wandbox-rs
An api binding for Wandbox in rust

Add this to your Cargo.toml
```toml
[dependencies]
wandbox = "0.1"
```


## Example
```rust
let wbox : Wandbox = Wandbox::new(None, None).await?;

let mut builder = crate::CompilationBuilder::new();
builder.target("gcc-6.3.0");
builder.options_str(vec!["-Wall", "-Werror"]);
builder.code("#include<iostream>\nint main()\n{\nstd::cout<<\"test\";\n}");
builder.build(&wbox)?;

let res = builder.dispatch().await.expect("Failed to lookup");
```


## License
This project is licensed under there LGPL v3 license. This license is available in LICENSE.txt