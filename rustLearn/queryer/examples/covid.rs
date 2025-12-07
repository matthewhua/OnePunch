use anyhow::Result;
use polars::prelude::*;
use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
    let data = reqwest::get(url).await?.text().await?;

    let df = CsvReader::new(Cursor::new(data))
        .infer_schema(Some(16))
        .finish()?;

    let filtered = df.filter(&df["new_cases"].gt(1000))?;
    
    let result = filtered
        .select((
            "location",
            "total_cases",
            "new_cases",
            "total_deaths",
            "new_deaths"
        ))?
        .head(Some(10));

    // 定义列宽
    const COL_WIDTH: usize = 20;
    
    // 打印表头
    println!("\x x x:");
    println!("{}", "=".repeat(COL_WIDTH * 5));
    
    // 打印列名
    for name in result.get_column_names() {
        print!("{:width$}", name, width = COL_WIDTH);
    }
    println!("\n{}", "-".repeat(COL_WIDTH * 5));

    // 打印数据行
    for i in 0..result.height() {
        if let Some(row) = result.get(i) {
            for value in row {
                // 格式化数值
                let formatted = match value {
                    AnyValue::Float64(f) => format!("{:.0}", f),  // 去除小数
                    AnyValue::Utf8(s) => s.to_string(),
                    _ => value.to_string(),
                };
                print!("{:width$}", formatted, width = COL_WIDTH);
            }
            println!();
        }
    }

    // 打印底部
    println!("{}", "=".repeat(COL_WIDTH * 5));
    println!("总计: {} 行数据", result.height());

    Ok(())
}
