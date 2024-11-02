// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors
//
//

/// include_proto! is a macro that includes the generated proto file from the OUT_DIR
/// use package name as the argument
/// ```no_run
/// include_proto!("lance.encodings");
/// ```
macro_rules! include_proto {
    ($package: tt) => {
        include!(concat!("generated/", $package, ".rs"));
    };
}

/// There some conflicts between encodings.proto and file.proto
/// So we need to include them separately
pub mod pb_encodings {
    // include the package lance.encodings from proto file encodings.proto
    include_proto!("lance.encodings");
}

pub mod pb {
    use lance_arrow::bfloat16::ARROW_EXT_NAME_KEY;
    use lance_core::datatypes::{
        Dictionary as CoreDictionary, Encoding as CoreEncoding, Field as CoreField, LogicalType,
    };
    use std::collections::HashMap;

    // include the package lance.file from proto file file.proto
    include_proto!("lance.file");
    // include the package lance.index.pb from proto file index.proto
    include_proto!("lance.index.pb");
    // include the package lance.table from proto file table.proto
    include_proto!("lance.table");

    impl From<&CoreDictionary> for Dictionary {
        fn from(d: &CoreDictionary) -> Self {
            Self {
                offset: d.offset as i64,
                length: d.length as i64,
            }
        }
    }

    impl From<&Dictionary> for CoreDictionary {
        fn from(proto: &Dictionary) -> Self {
            Self {
                offset: proto.offset as usize,
                length: proto.length as usize,
                values: None,
            }
        }
    }

    impl From<CoreEncoding> for Encoding {
        fn from(e: CoreEncoding) -> Self {
            match e {
                CoreEncoding::Plain => Self::Plain,
                CoreEncoding::VarBinary => Self::VarBinary,
                CoreEncoding::Dictionary => Self::Dictionary,
                CoreEncoding::RLE => Self::Rle,
            }
        }
    }
    impl From<&Field> for CoreField {
        fn from(field: &Field) -> Self {
            let mut lance_metadata: HashMap<String, String> = field
                .metadata
                .iter()
                .map(|(key, value)| {
                    let string_value = String::from_utf8_lossy(value).to_string();
                    (key.clone(), string_value)
                })
                .collect();
            if !field.extension_name.is_empty() {
                lance_metadata.insert(ARROW_EXT_NAME_KEY.to_string(), field.extension_name.clone());
            }
            Self {
                name: field.name.clone(),
                id: field.id,
                parent_id: field.parent_id,
                logical_type: LogicalType::from(field.logical_type.as_str()),
                metadata: lance_metadata,
                encoding: match field.encoding {
                    1 => Some(CoreEncoding::Plain),
                    2 => Some(CoreEncoding::VarBinary),
                    3 => Some(CoreEncoding::Dictionary),
                    4 => Some(CoreEncoding::RLE),
                    _ => None,
                },
                nullable: field.nullable,
                children: vec![],
                dictionary: field.dictionary.as_ref().map(CoreDictionary::from),
                storage_class: field.storage_class.parse().unwrap(),
            }
        }
    }

    impl From<&CoreField> for Field {
        fn from(field: &CoreField) -> Self {
            let pb_metadata = field
                .metadata
                .clone()
                .into_iter()
                .map(|(key, value)| (key, value.into_bytes()))
                .collect();
            Self {
                id: field.id,
                parent_id: field.parent_id,
                name: field.name.clone(),
                logical_type: field.logical_type.to_string(),
                encoding: match field.encoding {
                    Some(CoreEncoding::Plain) => 1,
                    Some(CoreEncoding::VarBinary) => 2,
                    Some(CoreEncoding::Dictionary) => 3,
                    Some(CoreEncoding::RLE) => 4,
                    _ => 0,
                },
                nullable: field.nullable,
                dictionary: field.dictionary.as_ref().map(Dictionary::from),
                metadata: pb_metadata,
                extension_name: field
                    .extension_name()
                    .map(|name| name.to_owned())
                    .unwrap_or_default(),
                r#type: 0,
                storage_class: field.storage_class().to_string(),
            }
        }
    }
}

pub mod pbfile {
    // include the package lance.file.v2 from proto file file2.proto
    include_proto!("lance.file.v2");
}
