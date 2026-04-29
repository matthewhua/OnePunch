use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置内置的 protoc 路径（无需系统安装 protoc）
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    println!("cargo:rerun-if-changed=proto");

    // 收集所有 .proto 文件
    let mut proto_files = Vec::new();
    for entry in fs::read_dir("proto")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|s| s == "proto").unwrap_or(false) {
            proto_files.push(path);
        }
    }

    // 编译 proto 文件，生成 Rust 结构体
    // prost 根据文件头的 syntax = "proto2"/"proto3" 自动判断语法版本
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&proto_files, &["proto"])?;

    // 从 proto2 文件中提取命令号，生成 cmd_generated.rs
    generate_cmd_table(&proto_files)?;

    Ok(())
}

/// 扫描所有 proto 文件，提取 `extend Base { optional XxxMsg ext = N; }` 中的命令号
///
/// 生成的文件路径：`$OUT_DIR/cmd_generated.rs`
/// 在 proto/src/lib.rs 中通过 `include!(concat!(env!("OUT_DIR"), "/cmd_generated.rs"))` 引入
fn generate_cmd_table(proto_files: &[std::path::PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    // 匹配 extend Base { ... } 块（处理跨行，先把换行替换成空格）
    let extend_block_re = Regex::new(r"extend\s+Base\s*\{([^}]*)\}")?;
    // 在块内匹配：optional MsgName ext = N;
    let field_re = Regex::new(r"optional\s+(\w+)\s+ext\s*=\s*(\d+)\s*;")?;

    // BTreeMap 保证按命令号有序输出
    let mut cmds: BTreeMap<u32, String> = BTreeMap::new();

    for path in proto_files {
        let content = fs::read_to_string(path)?;
        // 展平为单行，方便跨行匹配
        let flat = content.replace('\n', " ").replace('\r', "");

        for block_cap in extend_block_re.captures_iter(&flat) {
            let block = &block_cap[1];
            for field_cap in field_re.captures_iter(block) {
                let msg_name = field_cap[1].to_string();
                let cmd_id: u32 = field_cap[2].parse()?;
                cmds.insert(cmd_id, msg_name);
            }
        }
    }

    let out_dir = std::env::var("OUT_DIR")?;
    let dest = Path::new(&out_dir).join("cmd_generated.rs");

    let mut code = String::new();
    code.push_str("// 此文件由 build.rs 自动生成，请勿手动修改\n");
    code.push_str("// 来源：proto 文件中 `extend Base { optional XxxMsg ext = N; }` 定义\n\n");

    // ── GameCmd 枚举 ──────────────────────────────────────────────────────────
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("#[repr(u32)]\n");
    code.push_str("pub enum GameCmd {\n");
    code.push_str("    /// 未知命令号\n");
    code.push_str("    Unknown = 0,\n");
    for (id, name) in &cmds {
        code.push_str(&format!("    /// `{name}` (cmd = {id})\n"));
        code.push_str(&format!("    {name} = {id},\n"));
    }
    code.push_str("}\n\n");

    // ── From<u32> ─────────────────────────────────────────────────────────────
    code.push_str("impl From<u32> for GameCmd {\n");
    code.push_str("    fn from(id: u32) -> Self {\n");
    code.push_str("        match id {\n");
    for (id, name) in &cmds {
        code.push_str(&format!("            {id} => GameCmd::{name},\n"));
    }
    code.push_str("            _ => GameCmd::Unknown,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // ── From<GameCmd> for u32 / i32 ───────────────────────────────────────────
    code.push_str("impl From<GameCmd> for u32 {\n");
    code.push_str("    fn from(cmd: GameCmd) -> u32 { cmd as u32 }\n");
    code.push_str("}\n\n");
    code.push_str("impl From<GameCmd> for i32 {\n");
    code.push_str("    fn from(cmd: GameCmd) -> i32 { cmd as i32 }\n");
    code.push_str("}\n");

    fs::write(&dest, &code)?;
    println!("cargo:warning=cmd_generated.rs: {} commands extracted", cmds.len());

    Ok(())
}
