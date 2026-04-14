fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置内置的 protoc 路径
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    println!("cargo:rerun-if-changed=proto");

    let proto_files = &[
        "proto/service.proto", // 入口文件，它 import 了 Rpc.proto 和 World.proto
    ];

    // 配置 Tonic 构建
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        // 允许生成包含 serde 支持的代码（可选，但对 SLG 很有用）
        // .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(proto_files, &["proto"])?;

    Ok(())
}
