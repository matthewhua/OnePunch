use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let proto_dir = PathBuf::from("proto");
    let protoc_path = resolve_protoc_path()?;

    let mut inputs = fs::read_dir(&proto_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("proto"))
        .collect::<Vec<_>>();

    inputs.sort();

    for input in &inputs {
        println!("cargo:rerun-if-changed={}", input.display());
    }
    println!("cargo:rerun-if-changed={}", protoc_path.display());

    let input_strings = inputs
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    let mut codegen = protobuf_codegen::Codegen::new();
    codegen
        .protoc()
        .protoc_path(&protoc_path)
        .cargo_out_dir("pb")
        .includes(&[proto_dir.to_string_lossy().as_ref()])
        .inputs(input_strings.iter().map(|s| s.as_str()));

    codegen.run_from_script();
    Ok(())
}

fn resolve_protoc_path() -> Result<PathBuf, Box<dyn Error>> {
    let bundled = Path::new("..").join("..").join("protoc.exe");
    if bundled.exists() {
        Ok(bundled)
    } else {
        Ok(protoc_bin_vendored::protoc_bin_path()?)
    }
}
