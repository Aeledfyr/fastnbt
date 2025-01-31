# fastnbt project
[![Latest Version]][crates.io]
[![docs-shield]][docs]
[![Build Status]][actions]

[docs]: https://docs.rs/fastnbt/latest/fastnbt/index.html
[docs-shield]: https://img.shields.io/docsrs/fastnbt

[Build Status]:
https://img.shields.io/github/workflow/status/owengage/fastnbt/Rust/master
[actions]: https://github.com/owengage/fastnbt/actions?query=branch%3Amaster
[Latest Version]: https://img.shields.io/crates/v/fastnbt.svg
[crates.io]: https://crates.io/crates/fastnbt

Serde deserializer for *Minecraft: Java Edition's* NBT format, and in-browser
Rust-to-WASM powered Minecraft map renderer. Supports 1.18 down to 1.15
inclusive. Eventually want to support at least down to 1.12.

# Demo

Demo of Hermitcraft season 7 and more at [owengage.com/anvil](https://owengage.com/anvil/?world=hermitcraft7)

![alt rendered map](demo.png)  

This repository contains multiple related projects.

* [fastnbt](fastnbt/README.md): Fast serde deserializer for *Minecraft: Java
  Edition*'s NBT data format.
* fastanvil: For rendering Minecraft worlds to maps.
* fastnbt-tools: Various tools for NBT/Anvil, notably a map renderer.

Tries to support old chunks in worlds, but not
extracting textures from older versions due to the added
complexity it would require.

The `anvil` binary from `fastnbt-tools` can render your world leveraging all of
your CPU.

See [fastnbt's README](fastnbt/README.md) for performance comparison.

# Serde deserializer example

 This example demonstrates printing out a players inventory and ender chest contents from the [player dat
 files](https://minecraft.gamepedia.com/Player.dat_format) found in worlds. We
 * use serde's renaming attribute to have rustfmt conformant field names,
 * use lifetimes to save on string allocations, and 
 * use the `Value` type to deserialize a field we don't specify the exact
   structure of.

```rust
use fastnbt::error::Result;
use fastnbt::{de::from_bytes, Value};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct PlayerDat<'a> {
    data_version: i32,

    #[serde(borrow)]
    inventory: Vec<InventorySlot<'a>>,
    ender_items: Vec<InventorySlot<'a>>,
}

#[derive(Deserialize, Debug)]
struct InventorySlot<'a> {
    id: &'a str,        // We avoid allocating a string here.
    tag: Option<Value>, // Also get the less structured properties of the object.

    // We need to rename fields a lot.
    #[serde(rename = "Count")]
    count: i8,
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file = std::fs::File::open(args[0].clone()).unwrap();

    // Player dat files are compressed with GZip.
    let mut decoder = GzDecoder::new(file);
    let mut data = vec![];
    decoder.read_to_end(&mut data).unwrap();

    let player: Result<PlayerDat> = from_bytes(data.as_slice());

    println!("{:#?}", player);
}
```

# Development priorities

These are the proirities for the project when it comes to development. Ideally
we never sacrifice an earlier priority for the sake of a later one.

1. Correctness. Worlds are rendered as accurately as possible.
2. Speed. Worlds are rendered as fast as possible.
3. Memory. Worlds are rendered without sucking up RAM.

## Usage

For the libraries

```toml
[dependencies]
fastnbt = "1"
fastanvil = "0.22"
```

For the `anvil` executable

```bash
cargo install fastnbt-tools
```
