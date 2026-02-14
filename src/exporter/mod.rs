use crate::core::terrain::TerrainGrid;
use anyhow::Result;
use lz4_java_wrc::Lz4BlockOutput;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

const COMPRESSION_LZ4: u8 = 4;
const SECTOR_SIZE: u64 = 4096;
const DATA_VERSION: i32 = 4671;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ChunkRoot {
    #[serde(rename = "DataVersion")]
    data_version: i32,
    #[serde(rename = "xPos")]
    x_pos: i32,
    #[serde(rename = "zPos")]
    z_pos: i32,
    #[serde(rename = "yPos")]
    y_pos: i32,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "sections")]
    sections: Vec<Section>,
}

#[derive(Serialize)]
struct Section {
    #[serde(rename = "Y")]
    y: i8,
    block_states: BlockStates,
    biomes: Biomes,
}

#[derive(Serialize)]
struct BlockStates {
    palette: Vec<BlockStatePalette>,
    #[serde(
        rename = "data",
        with = "na_nbt::long_array"
    )]
    data: Vec<i64>,
}

#[derive(Serialize)]
struct BlockStatePalette {
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Serialize)]
struct Biomes {
    palette: Vec<String>,
}

struct ExportConfig {
    world_min_y: i32,
    world_height: i32,
    vertical_offset: f32,
}

pub fn generate_world(output_dir: &Path, grid: &TerrainGrid) -> Result<()> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let config = calculate_export_config(grid.min_elevation, grid.max_elevation);

    println!(
        "Origin Height: {:.2}m ~ {:.2}m (diff: {:.2}m)",
        grid.min_elevation,
        grid.max_elevation,
        grid.max_elevation - grid.min_elevation
    );
    println!("Offset: -{:.2}m", config.vertical_offset);
    println!(
        "Mapped Height: {:.2} ~ {:.2}",
        grid.min_elevation - config.vertical_offset,
        grid.max_elevation - config.vertical_offset
    );
    println!(
        "Min Y = {}, Height = {}, Top Y = {}",
        config.world_min_y,
        config.world_height,
        config.world_min_y + config.world_height - 1
    );

    let limit_max = (config.world_min_y + config.world_height - 1) as f32;
    let mapped_max = grid.max_elevation - config.vertical_offset;
    if mapped_max > limit_max {
        panic!("Map height is bigger than max height");
    } else if config.world_height > 384 || config.world_min_y < -64 {
        println!("Warn: Map height is bigger than 384, please install Higher Heights Datapack");
    }

    const REGION_BLOCK_SIZE: usize = 512;
    let regions_x = grid.width.div_ceil(REGION_BLOCK_SIZE);
    let regions_z = grid.height.div_ceil(REGION_BLOCK_SIZE);

    println!("Will Gen Region: X[0..{}] Z[0..{}]", regions_x, regions_z);

    for rx in 0..regions_x as i32 {
        for rz in 0..regions_z as i32 {
            println!("Writing: r.{}.{}.mca", rx, rz);
            write_mc_lz4_region(output_dir, rx, rz, grid, &config)?;
        }
    }

    Ok(())
}

fn calculate_export_config(min_ele: f32, max_ele: f32) -> ExportConfig {
    const ABS_MIN_Y: i32 = -2032;
    const MAX_CAPACITY: i32 = 4064;
    const ABS_MAX_Y: i32 = ABS_MIN_Y + MAX_CAPACITY;

    let total_span = max_ele - min_ele;

    let mut target_min_y = -64;

    if total_span > 384.0 {
        let target_max_y = ABS_MAX_Y - 16;

        let calculated_min_y = target_max_y as f32 - total_span;

        if calculated_min_y < ABS_MIN_Y as f32 {
            target_min_y = ABS_MIN_Y;
        } else {
            target_min_y = (calculated_min_y as i32 / 16) * 16;
        }
    }

    let vertical_offset = min_ele - target_min_y as f32;

    let mapped_top = max_ele - vertical_offset;
    let req_top_y = (mapped_top.ceil() as i32 + 15) / 16 * 16;

    let mut height = req_top_y - target_min_y;

    height = height.clamp(384, MAX_CAPACITY);

    ExportConfig {
        world_min_y: target_min_y,
        world_height: height,
        vertical_offset,
    }
}

fn write_mc_lz4_region(
    dir: &Path,
    rx: i32,
    rz: i32,
    grid: &TerrainGrid,
    config: &ExportConfig,
) -> Result<()> {
    let path = dir.join(format!("r.{}.{}.mca", rx, rz));
    let mut file = File::create(path)?;

    file.write_all(&[0u8; 8192])?;

    let mut locations = vec![0u32; 1024];
    let mut current_sector_offset = 2u32;

    for cz in 0..32 {
        for cx in 0..32 {
            let global_x = (rx * 512 + cx * 16) as usize;
            let global_z = (rz * 512 + cz * 16) as usize;

            let chunk_data = build_chunk_struct(grid, global_x, global_z, config);

            let mut uncompressed_bytes = Vec::with_capacity(4096);
            na_nbt::to_writer_be(&mut uncompressed_bytes, &chunk_data)?;

            let mut compressed_data = Vec::with_capacity(uncompressed_bytes.len());
            {
                let mut encoder = Lz4BlockOutput::new(&mut compressed_data);
                encoder.write_all(&uncompressed_bytes)?;
                encoder.flush()?;
            }

            let payload_len = compressed_data.len() as u32 + 1;
            file.write_all(&payload_len.to_be_bytes())?;
            file.write_all(&[COMPRESSION_LZ4])?;
            file.write_all(&compressed_data)?;

            let total_written = 4 + 1 + compressed_data.len();
            let padding = (SECTOR_SIZE as usize - (total_written % SECTOR_SIZE as usize))
                % SECTOR_SIZE as usize;
            if padding > 0 {
                file.write_all(&vec![0u8; padding])?;
            }

            let sectors_used = (total_written + padding) / SECTOR_SIZE as usize;
            let loc = (current_sector_offset << 8) | (sectors_used as u32);
            locations[(cz * 32 + cx) as usize] = loc;
            current_sector_offset += sectors_used as u32;
        }
    }

    file.seek(SeekFrom::Start(0))?;
    for loc in locations {
        file.write_all(&loc.to_be_bytes())?;
    }

    Ok(())
}

fn build_chunk_struct(
    grid: &TerrainGrid,
    gx: usize,
    gz: usize,
    config: &ExportConfig,
) -> ChunkRoot {
    let min_y = config.world_min_y;
    let max_y = min_y + config.world_height;

    let mut height_map = [min_y; 256];
    let mut chunk_min_h = i32::MAX;
    let mut chunk_max_h = i32::MIN;

    for z in 0..16 {
        for x in 0..16 {
            let cur_gx = gx + x;
            let cur_gz = gz + z;

            let h = if cur_gx < grid.width && cur_gz < grid.height {
                let idx = cur_gz * grid.width + cur_gx;
                if idx < grid.elevation.len() {
                    let val = grid.elevation[idx];
                    if val.is_nan() {
                        min_y
                    } else {
                        (val - config.vertical_offset).floor() as i32
                    }
                } else {
                    min_y
                }
            } else {
                min_y
            };

            height_map[z * 16 + x] = h;
            if h < chunk_min_h {
                chunk_min_h = h;
            }
            if h > chunk_max_h {
                chunk_max_h = h;
            }
        }
    }

    let mut sections = Vec::new();
    let min_section_idx = min_y >> 4;
    let max_section_idx = (max_y >> 4) - 1;

    for sy in min_section_idx..=max_section_idx {
        let base_y = sy * 16;
        let top_y = base_y + 15;

        if base_y > chunk_max_h {
            continue;
        }

        if top_y <= chunk_min_h {
            let palette = vec![BlockStatePalette {
                name: "minecraft:stone".to_string(),
            }];
            sections.push(Section {
                y: sy as i8,
                block_states: BlockStates {
                    palette,
                    data: vec![],
                },
                biomes: Biomes {
                    palette: vec!["minecraft:plains".to_string()],
                },
            });
            continue;
        }

        let palette = vec![
            BlockStatePalette {
                name: "minecraft:air".to_string(),
            },
            BlockStatePalette {
                name: "minecraft:stone".to_string(),
            },
        ];

        let mut block_indices = Vec::with_capacity(4096);

        for y in 0..16 {
            let abs_y = base_y + y;
            for z in 0..16 {
                for x in 0..16 {
                    let h = height_map[z * 16 + x];
                    if abs_y <= h {
                        block_indices.push(1); // Stone
                    } else {
                        block_indices.push(0); // Air
                    }
                }
            }
        }

        let packed_data = pack_states(&block_indices, 4);

        sections.push(Section {
            y: sy as i8,
            block_states: BlockStates {
                palette,
                data: packed_data,
            },
            biomes: Biomes {
                palette: vec!["minecraft:plains".to_string()],
            },
        });
    }

    ChunkRoot {
        data_version: DATA_VERSION,
        x_pos: (gx / 16) as i32,
        z_pos: (gz / 16) as i32,
        y_pos: min_y,
        status: "minecraft:features".to_string(),
        sections,
    }
}

fn pack_states(states: &[usize], bits_per_block: usize) -> Vec<i64> {
    let blocks_per_long = 64 / bits_per_block;
    let long_count = states.len().div_ceil(blocks_per_long);
    let mut data = vec![0i64; long_count];

    for (i, &state) in states.iter().enumerate() {
        let long_index = i / blocks_per_long;
        let sub_index = i % blocks_per_long;
        let bit_offset = sub_index * bits_per_block;

        data[long_index] |= (state as i64) << bit_offset;
    }
    data
}
