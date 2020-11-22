use std::{collections::HashMap, convert::TryFrom};

use byteorder::{BigEndian, ReadBytesExt};
use serde::Deserialize;

use crate::{bits_per_block, PackedBits};

use super::biome::Biome;

/// A Minecraft chunk.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Chunk<'a> {
    pub data_version: i32,

    #[serde(borrow)]
    pub level: Level<'a>,
}

/// A level describes the contents of the chunk in the world.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Level<'a> {
    #[serde(rename = "xPos")]
    pub x_pos: i32,

    #[serde(rename = "zPos")]
    pub z_pos: i32,

    pub biomes: Option<&'a [u8]>,

    #[serde(borrow)]
    pub heightmaps: Option<Heightmaps<'a>>,

    // Old chunk formats can store a plain heightmap in an IntArray here.
    #[serde(rename = "HeightMap")]
    pub old_heightmap: Option<Vec<i32>>,

    /// Can be empty if the chunk hasn't been generated properly yet.
    pub sections: Option<Vec<Section<'a>>>,

    // Status of the chunk. Typically anything except 'full' means the chunk
    // hasn't been fully generated yet. We use this to skip chunks on map edges
    // that haven't been fully generated yet.
    pub status: &'a str,

    // Maps the y value from each section to the index in the `sections` field.
    // Makes it quicker to find the correct section when all you have is the height.
    #[serde(skip)]
    #[serde(default)]
    sec_map: HashMap<i8, usize>,
}

/// Various heightmaps kept up to date by Minecraft.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Heightmaps<'a> {
    #[serde(borrow)]
    pub motion_blocking: Option<PackedBits<'a>>,
    pub motion_blocking_no_leaves: Option<PackedBits<'a>>,
    pub ocean_floor: Option<PackedBits<'a>>,
    pub world_surface: Option<PackedBits<'a>>,

    #[serde(skip)]
    unpacked_motion_blocking: Option<[u16; 16 * 16]>,
}

/// A vertical section of a chunk (ie a 16x16x16 block cube)
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section<'a> {
    pub y: i8,

    #[serde(borrow)]
    pub block_states: Option<PackedBits<'a>>,

    #[serde(default)]
    pub palette: Vec<Block<'a>>,

    // Perhaps a little large to potentially end up on the stack? 8 KiB.
    #[serde(skip)]
    unpacked_states: Option<[u16; 16 * 16 * 16]>,
}

/// A block within the world.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Block<'a> {
    pub name: &'a str,

    #[serde(default)]
    pub properties: HashMap<&'a str, &'a str>,
}

impl<'a> Chunk<'a> {
    pub fn recalculate_heightmap(&mut self) {
        let mut map = [0; 256];
        for z in 0..16 {
            for x in 0..16 {
                // start at top until we hit a non-air block.
                for i in 0..255 {
                    let y = 256 - i;
                    let block = self.block(x, y - 1, z);
                    if block.is_none() {
                        continue;
                    }

                    if !["minecraft:air", "minecraft:cave_air"]
                        .as_ref()
                        .contains(&block.unwrap().name)
                    {
                        map[z * 16 + x] = y as u16;
                        break;
                    }
                }
            }
        }

        match self.level.heightmaps.as_mut() {
            Some(m) => m.unpacked_motion_blocking = Some(map),
            None => {}
        }
    }

    pub fn block(&mut self, x: usize, y: usize, z: usize) -> Option<&Block> {
        let sec = self.get_section_for_y(y)?;

        if (sec.y as usize) * 16 > y {}
        let sec_y = y - sec.y as usize * 16;
        let state_index = (sec_y as usize * 16 * 16) + z * 16 + x;

        if sec.unpacked_states == None {
            let bits_per_item = bits_per_block(sec.palette.len());
            sec.unpacked_states = Some([0; 16 * 16 * 16]);

            let buf = sec.unpacked_states.as_mut()?;

            sec.block_states
                .as_ref()?
                .unpack_into(bits_per_item, &mut buf[..]);
        }

        let pal_index = sec.unpacked_states.as_ref()?[state_index] as usize;
        sec.palette.get(pal_index)
    }

    pub fn height_of(&mut self, x: usize, z: usize) -> Option<usize> {
        let ref mut maps = self.level.heightmaps;

        match maps {
            Some(maps) => {
                if maps.unpacked_motion_blocking == None {
                    maps.unpacked_motion_blocking = Some([0; 256]);
                    let buf = maps.unpacked_motion_blocking.as_mut()?;
                    maps.motion_blocking
                        .as_ref()?
                        .unpack_heights_into(&mut buf[..]);
                }

                Some(maps.unpacked_motion_blocking.as_ref()?[z * 16 + x] as usize)
            }
            None => self // Older style heightmap found. Much simpler, just an int per column.
                .level
                .old_heightmap
                .as_ref()
                .map(|v| v[z * 16 + x] as usize),
        }
    }

    pub fn biome_of(&self, x: usize, _y: usize, z: usize) -> Option<Biome> {
        // TODO: Take into account height. For overworld this doesn't matter (at least not yet)
        // TODO: Make use of data version?

        // For biome len of 1024,
        //  it's 4x4x4 sets of blocks stored by z then x then y (+1 moves one in z)
        //  for overworld theres no vertical chunks so it looks like only first 16 values are used.
        // For biome len of 256, it's chunk 1x1 columns stored z then x.

        let biomes = self.level.biomes?;

        if biomes.len() == 1024 * 4 {
            // Minecraft 1.16
            let i = 4 * ((z / 4) * 4 + (x / 4));
            let biome = (&biomes[i..]).read_i32::<BigEndian>().ok()?;

            Biome::try_from(biome).ok()
        } else if biomes.len() == 256 * 4 {
            // Minecraft 1.15 (and past?)
            let i = 4 * (z * 16 + x);
            let biome = (&biomes[i..]).read_i32::<BigEndian>().ok()?;
            Biome::try_from(biome).ok()
        } else {
            None
        }
    }

    fn calculate_sec_map(&mut self) {
        let map = &mut self.level.sec_map;

        for (i, sec) in self.level.sections.iter().flatten().enumerate() {
            map.insert(sec.y, i);
        }
    }

    fn get_section_for_y(&mut self, y: usize) -> Option<&mut Section<'a>> {
        if self.level.sections.as_ref()?.is_empty() {
            return None;
        }

        if self.level.sec_map.is_empty() {
            self.calculate_sec_map();
        }

        let containing_section_y = y / 16;
        let section_index = self.level.sec_map.get(&(containing_section_y as i8))?;

        let sec = self.level.sections.as_mut()?.get_mut(*section_index);
        sec
    }
}

impl<'a> Block<'a> {
    /// Creates a string of the format "id|prop1=val1,prop2=val2". The
    /// properties are ordered lexigraphically. This somewhat matches the way
    /// Minecraft stores variants in blockstates, but with the block ID/name
    /// prepended.
    pub fn encoded_description(&self) -> String {
        let mut id = self.name.to_string() + "|";
        let mut sep = "";

        let mut props = self
            .properties
            .iter()
            .filter(|(k, _)| **k != "waterlogged") // TODO: Handle water logging. See note below
            .collect::<Vec<_>>();

        // need to sort the properties for a consistent ID
        props.sort();

        for (k, v) in props {
            id = id + sep + k + "=" + v;
            sep = ",";
        }

        id

        // Note: If we want to handle water logging, we're going to have to
        // remove the filter here and handle it in whatever parses the encoded
        // ID itself. This will probably be pretty ugly. It can probably be
        // contained in the palette generation code entirely, so shouldn't
        // impact speed to hard.
    }
}
