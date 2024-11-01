// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

use std::io::Result;
fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=protos");
    let mut prost_build = prost_build::Config::new();
    prost_build.protoc_arg("--experimental_allow_proto3_optional");
    prost_build.extern_path(".lance.file", "crate::pb");
    prost_build.extern_path(".lance.encodings", "crate::pb_encodings");
    prost_build.enable_type_names();
    prost_build.compile_protos(
        &[
            "protos/file.proto",
            "protos/file2.proto",
            "protos/encodings.proto",
            "protos/table.proto",
            "protos/transaction.proto",
            "protos/rowids.proto",
            "protos/index.proto",
        ],
        &["protos"],
    )?;
    Ok(())
}
