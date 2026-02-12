use super::geo_utils::extract_bounds_async;
use crate::scanner::path_utils::normalize_path;
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::scanner::types::SoilTile;

pub async fn scan(root: PathBuf) -> Result<Vec<SoilTile>> {
    let p_sand_top = root.join("sand/0-5");
    let p_sand_sub = root.join("sand/30-60");
    let p_clay_top = root.join("clay/0-5");
    let p_clay_sub = root.join("clay/30-60");
    let p_soc_top = root.join("soc/0-5");
    let p_ph_top = root.join("phh2o/0-5");
    let p_ph_sub = root.join("phh2o/30-60");

    let map_sand_top = scan_single_layer(&p_sand_top, "Sand Top")?;
    let map_sand_sub = scan_single_layer(&p_sand_sub, "Sand Sub")?;
    let map_clay_top = scan_single_layer(&p_clay_top, "Clay Top")?;
    let map_clay_sub = scan_single_layer(&p_clay_sub, "Clay Sub")?;
    let map_soc_top = scan_single_layer(&p_soc_top, "SOC Top")?;
    let map_ph_top = scan_single_layer(&p_ph_top, "pH Top")?;
    let map_ph_sub = scan_single_layer(&p_ph_sub, "pH Sub")?;

    let bundles = align_layers(
        map_sand_top,
        map_sand_sub,
        map_clay_top,
        map_clay_sub,
        map_soc_top,
        map_ph_top,
        map_ph_sub,
    )
    .await;

    Ok(bundles)
}

type LayerMap = HashMap<String, PathBuf>;

fn scan_single_layer(dir: &Path, layer_name: &str) -> Result<LayerMap> {
    if !dir.exists() {
        return Err(anyhow!(
            "Directory not found for layer [{}]: {}",
            layer_name,
            normalize_path(dir),
        ));
    }

    let mut map = HashMap::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let is_tif = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase() == "tif")
            .unwrap_or(false);

        if !is_tif {
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
            map.insert(file_name.to_string(), path.to_path_buf());
        }
    }

    Ok(map)
}

async fn align_layers(
    sand_top: LayerMap,
    sand_sub: LayerMap,
    clay_top: LayerMap,
    clay_sub: LayerMap,
    soc_top: LayerMap,
    ph_top: LayerMap,
    ph_sub: LayerMap,
) -> Vec<SoilTile> {
    let candidates: HashSet<String> = sand_top.keys().cloned().collect();
    let mut bundles = Vec::new();

    for id in candidates {
        let maybe_bundle = (async || {
            let p1 = sand_top.get(&id)?;
            let p2 = sand_sub.get(&id)?;
            let p3 = clay_top.get(&id)?;
            let p4 = clay_sub.get(&id)?;
            let p5 = soc_top.get(&id)?;
            let p6 = ph_top.get(&id)?;
            let p7 = ph_sub.get(&id)?;

            let bounds = extract_bounds_async(p1).await.ok()?;

            Some(SoilTile {
                id: id.clone(),
                bounds,
                sand_top: p1.clone(),
                sand_sub: p2.clone(),
                clay_top: p3.clone(),
                clay_sub: p4.clone(),
                soc_top: p5.clone(),
                ph_top: p6.clone(),
                ph_sub: p7.clone(),
            })
        })()
        .await;

        if let Some(bundle) = maybe_bundle {
            bundles.push(bundle);
        }
    }

    bundles.sort_by(|a, b| a.id.cmp(&b.id));

    bundles
}
