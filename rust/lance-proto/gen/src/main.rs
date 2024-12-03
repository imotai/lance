// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

use std::io::Result;

// generated files will be placed in the OUT_DIR
// build only if the proto files have changed
const OUT_DIR: &str = "src/generated";

fn build_protos(proto_files: &[&str], dirs: &[&str], extern_paths: &[(&str, &str)]) -> Result<()> {
    let mut prost_build = prost_build::Config::new();
    extern_paths.iter().for_each(|(extern_path, crate_path)| {
        prost_build.extern_path(*extern_path, *crate_path);
    });
    prost_build.protoc_arg("--experimental_allow_proto3_optional");
    prost_build.enable_type_names();
    prost_build.out_dir(OUT_DIR);
    prost_build.compile_protos(proto_files, dirs)?;
    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=protos");
    // build the encodings proto for extern_paths
    build_protos(&["protos/encodings.proto"], &["protos"], &[])?;
    // build the file proto  for extern_paths
    build_protos(&["protos/file.proto"], &["protos"], &[])?;
    build_protos(&["protos/file2.proto"], &["protos"], &[])?;

    build_protos(
        &[
            "protos/table.proto",
            "protos/transaction.proto",
            "protos/rowids.proto",
            "protos/index.proto",
        ],
        &["protos"],
        &[
            (".lance.file", "crate::pb"),
            (".lance.encodings", "crate::pb_encodings"),
        ],
    )?;

    Ok(())
}
