// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

pub use lance_proto::pb;
pub use lance_proto::pbfile;

pub mod metadata;

/// These version/magic values are written at the end of Lance files (e.g. versions/1.version)
pub const MAJOR_VERSION: i16 = 0;
pub const MINOR_VERSION: i16 = 2;
pub const MAGIC: &[u8; 4] = b"LANC";
