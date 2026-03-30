mod web;

use anyhow::{Context, Result};
use clap::Parser;
use lopdf::Document;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pdf-split", about = "PDF 分割工具 - 支持命令行和网页模式")]
struct Args {
    /// 输入 PDF 文件路径 (命令行模式)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// 输出目录 (命令行模式，默认为当前目录)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// 要提取的页码范围，如 "1-3,5,7-9"
    #[arg(short, long)]
    pages: Option<String>,

    /// 每份分割的页数 (与 --pages 互斥)
    #[arg(short = 'n', long)]
    split_every: Option<usize>,

    /// 启动网页服务器模式
    #[arg(short, long, default_value_t = false)]
    web: bool,

    /// 网页服务器监听端口
    #[arg(long, default_value_t = 3000)]
    port: u16,
}

/// 解析页码范围字符串，如 "1-3,5,7-9" -> [1,2,3,5,7,8,9]
pub fn parse_page_ranges(input: &str, max_page: u32) -> Result<Vec<u32>> {
    let mut pages = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() != 2 {
                anyhow::bail!("无效的页码范围: {}", part);
            }
            let start: u32 = bounds[0].trim().parse().context("无效的起始页码")?;
            let end: u32 = bounds[1].trim().parse().context("无效的结束页码")?;
            if start < 1 || end > max_page || start > end {
                anyhow::bail!("页码范围超出边界: {}-{} (总页数: {})", start, end, max_page);
            }
            for p in start..=end {
                pages.push(p);
            }
        } else {
            let page: u32 = part.parse().context("无效的页码")?;
            if page < 1 || page > max_page {
                anyhow::bail!("页码超出范围: {} (总页数: {})", page, max_page);
            }
            pages.push(page);
        }
    }
    pages.sort();
    pages.dedup();
    Ok(pages)
}

/// 从 PDF 中提取指定页码，生成新的 PDF 文档字节流
/// 注意：这是真正的分割——通过删除不需要的页面来重建文档，
/// 而不是简单地隐藏页面。
pub fn extract_pages(pdf_data: &[u8], pages: &[u32]) -> Result<Vec<u8>> {
    let mut doc = Document::load_mem(pdf_data).context("无法加载 PDF 文件")?;
    let total_pages = doc.get_pages().len() as u32;

    // 验证页码
    for &p in pages {
        if p < 1 || p > total_pages {
            anyhow::bail!("页码 {} 超出范围 (总页数: {})", p, total_pages);
        }
    }

    // 计算需要删除的页码（所有不在 pages 列表中的页码）
    let pages_to_delete: Vec<u32> = (1..=total_pages)
        .filter(|p| !pages.contains(p))
        .collect();

    // 使用 lopdf 的 delete_pages 真正移除不需要的页面
    doc.delete_pages(&pages_to_delete);

    let mut output = Vec::new();
    doc.save_to(&mut output).context("无法保存分割后的 PDF")?;

    Ok(output)
}

/// 按固定页数分割 PDF 为多个子文件
pub fn split_by_count(pdf_data: &[u8], pages_per_split: usize) -> Result<Vec<Vec<u8>>> {
    let doc = Document::load_mem(pdf_data).context("无法加载 PDF 文件")?;
    let total_pages = doc.get_pages().len() as u32;

    if pages_per_split == 0 {
        anyhow::bail!("每份页数必须大于 0");
    }

    let mut results = Vec::new();
    let mut start = 1u32;

    while start <= total_pages {
        let end = std::cmp::min(start + pages_per_split as u32 - 1, total_pages);
        let page_range: Vec<u32> = (start..=end).collect();
        let chunk = extract_pages(pdf_data, &page_range)?;
        results.push(chunk);
        start = end + 1;
    }

    Ok(results)
}

/// 获取 PDF 文件的总页数
pub fn get_page_count(pdf_data: &[u8]) -> Result<u32> {
    let doc = Document::load_mem(pdf_data).context("无法加载 PDF 文件")?;
    Ok(doc.get_pages().len() as u32)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.web {
        // 网页服务器模式
        println!("🌐 启动 PDF 分割器网页服务...");
        println!("📎 访问 http://127.0.0.1:{}", args.port);
        web::start_server(args.port).await?;
    } else {
        // 命令行模式
        let input = args.input.context("命令行模式需要指定 --input 参数")?;
        let pdf_data = std::fs::read(&input)
            .with_context(|| format!("无法读取文件: {}", input.display()))?;

        let total_pages = get_page_count(&pdf_data)?;
        println!("📄 PDF 文件: {} (共 {} 页)", input.display(), total_pages);

        let output_dir = args.output.unwrap_or_else(|| PathBuf::from("."));
        std::fs::create_dir_all(&output_dir)
            .with_context(|| format!("无法创建输出目录: {}", output_dir.display()))?;

        if let Some(pages_str) = &args.pages {
            // 指定页码范围模式
            let pages = parse_page_ranges(pages_str, total_pages)?;
            println!("📋 提取页面: {:?}", pages);

            let result = extract_pages(&pdf_data, &pages)?;
            let stem = input.file_stem().unwrap().to_string_lossy();
            let output_path = output_dir.join(format!("{}_pages_{}.pdf", stem, pages_str.replace(',', "_")));

            std::fs::write(&output_path, result)
                .with_context(|| format!("无法写入文件: {}", output_path.display()))?;
            println!("✅ 已保存到: {}", output_path.display());
        } else if let Some(n) = args.split_every {
            // 按页数分割模式
            println!("✂️  每 {} 页分割一份", n);
            let chunks = split_by_count(&pdf_data, n)?;

            for (i, chunk) in chunks.iter().enumerate() {
                let stem = input.file_stem().unwrap().to_string_lossy();
                let output_path = output_dir.join(format!("{}_part_{}.pdf", stem, i + 1));
                std::fs::write(&output_path, chunk)
                    .with_context(|| format!("无法写入文件: {}", output_path.display()))?;
                println!("  📄 Part {}: {}", i + 1, output_path.display());
            }
            println!("✅ 分割完成，共 {} 份", chunks.len());
        } else {
            anyhow::bail!("请指定 --pages 或 --split-every 参数，或使用 --web 启动网页模式");
        }
    }

    Ok(())
}
