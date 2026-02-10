use anyhow::Result;
use std::path::PathBuf;
use geo::{Rect, Coord};
use scanner::DataCatalog;

mod scanner;
mod sampler;

#[tokio::main]
async fn main() {
    if let Err(e) = logic().await {
        eprintln!("{e}");
    }
}

async fn logic() -> Result<()> {
    let root = PathBuf::from("datasets");

    println!("开始扫描数据集: {:?}", root);

    let catalog = DataCatalog::scan(root).await?;

    println!("扫描完成:");
    println!(" - ALOS Palsar 片段: {} 个", catalog.alos.len());
    println!(" - ESA WorldCover 片段: {} 个", catalog.esa.len());
    println!(" - SoilGrids 片段: {} 个", catalog.soil.len());

    let roi = Rect::new(
        Coord { x: 94.02376, y: 30.15698 },
        Coord { x: 93.84993, y: 29.97956 },
        // 94.02376,30.15698,93.84993,29.97956
    );

    println!("\n正在校验 ROI 覆盖率...");
    let report = catalog.check_coverage(roi);

    if report.is_ready {
        println!("数据层已完整覆盖目标区域");
    } else {
        println!("数据缺失或覆盖不足");
        for detail in report.details {
            println!(" - {}", detail);
        }
    }

    Ok(())
}