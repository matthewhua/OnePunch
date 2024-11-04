use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use crate::pb::{filter, resize, ImageSpec, Spec};

mod pb;
mod server1;
mod server2;
mod engine;

fn main() {
    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["abi.proto"], &["."])
        .unwrap()
}




