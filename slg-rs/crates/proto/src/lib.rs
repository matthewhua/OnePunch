pub mod slg {
    // 包含由 tonic-build 生成的内容
    tonic::include_proto!("slg");
}

// 重新导出常用类型
pub use slg::*;

/// 自动生成的命令号枚举（来自 build.rs 解析 proto 文件）
/// 包含：GameCmd 枚举、From<u32>、From<GameCmd> for u32/i32
pub mod cmd {
    include!(concat!(env!("OUT_DIR"), "/cmd_generated.rs"));
}
