pub mod slg {
    // 包含由 tonic-build 生成的内容
    tonic::include_proto!("slg");
}

// 可以在这里导出常用的类型方便调用
pub use slg::*;
