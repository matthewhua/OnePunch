fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置内置的 protoc 路径
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    println!("cargo:rerun-if-changed=proto");

    // 获取 proto 目录下所有的 .proto 文件
    let mut proto_files = Vec::new();
    for entry in std::fs::read_dir("proto")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|s| s == "proto").unwrap_or(false) {
            proto_files.push(path);
        }
    }

    // 配置 Tonic 构建
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&proto_files, &["proto"])?;

    Ok(())
}
