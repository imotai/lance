// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

use std::collections::HashMap;

use arrow_schema::DataType;
use async_recursion::async_recursion;
use lance_arrow::DataTypeExt;
use lance_core::datatypes::{Field, Schema};
use lance_core::{Error, Result};
use lance_io::traits::Reader;
use lance_io::utils::{read_binary_array, read_fixed_stride_array};
use snafu::{location, Location};

use crate::format::pb;

pub struct Fields(pub Vec<pb::Field>);

impl From<&Field> for Fields {
    fn from(field: &Field) -> Self {
        let mut protos = vec![pb::Field::from(field)];
        protos.extend(field.children.iter().flat_map(|val| Self::from(val).0));
        Self(protos)
    }
}

/// Convert list of protobuf `Field` to a Schema.
impl From<&Fields> for Schema {
    fn from(fields: &Fields) -> Self {
        let mut schema = Self {
            fields: vec![],
            metadata: HashMap::default(),
        };

        fields.0.iter().for_each(|f| {
            if f.parent_id == -1 {
                schema.fields.push(Field::from(f));
            } else {
                let parent = schema.mut_field_by_id(f.parent_id).unwrap();
                parent.children.push(Field::from(f));
            }
        });

        schema
    }
}

pub struct FieldsWithMeta {
    pub fields: Fields,
    pub metadata: HashMap<String, Vec<u8>>,
}

/// Convert list of protobuf `Field` and Metadata to a Schema.
impl From<FieldsWithMeta> for Schema {
    fn from(fields_with_meta: FieldsWithMeta) -> Self {
        let lance_metadata = fields_with_meta
            .metadata
            .into_iter()
            .map(|(key, value)| {
                let string_value = String::from_utf8_lossy(&value).to_string();
                (key, string_value)
            })
            .collect();

        let schema_with_fields = Self::from(&fields_with_meta.fields);
        Self {
            fields: schema_with_fields.fields,
            metadata: lance_metadata,
        }
    }
}

/// Convert a Schema to a list of protobuf Field.
impl From<&Schema> for Fields {
    fn from(schema: &Schema) -> Self {
        let mut protos = vec![];
        schema.fields.iter().for_each(|f| {
            protos.extend(Self::from(f).0);
        });
        Self(protos)
    }
}

/// Convert a Schema to a list of protobuf Field and Metadata
impl From<&Schema> for FieldsWithMeta {
    fn from(schema: &Schema) -> Self {
        let fields = schema.into();
        let metadata = schema
            .metadata
            .clone()
            .into_iter()
            .map(|(key, value)| (key, value.into_bytes()))
            .collect();
        Self { fields, metadata }
    }
}

#[async_recursion]
async fn load_field_dictionary<'a>(field: &mut Field, reader: &dyn Reader) -> Result<()> {
    if let DataType::Dictionary(_, value_type) = field.data_type() {
        assert!(field.dictionary.is_some());
        if let Some(dict_info) = field.dictionary.as_mut() {
            use DataType::*;
            match value_type.as_ref() {
                _ if value_type.is_binary_like() => {
                    dict_info.values = Some(
                        read_binary_array(
                            reader,
                            value_type.as_ref(),
                            true, // Empty values are null
                            dict_info.offset,
                            dict_info.length,
                            ..,
                        )
                        .await?,
                    );
                }
                Int8 | Int16 | Int32 | Int64 | UInt8 | UInt16 | UInt32 | UInt64 => {
                    dict_info.values = Some(
                        read_fixed_stride_array(
                            reader,
                            value_type.as_ref(),
                            dict_info.offset,
                            dict_info.length,
                            ..,
                        )
                        .await?,
                    );
                }
                _ => {
                    return Err(Error::Schema {
                        message: format!(
                            "Does not support {} as dictionary value type",
                            value_type
                        ),
                        location: location!(),
                    });
                }
            }
        } else {
            panic!("Should not reach here: dictionary field does not load dictionary info")
        }
        Ok(())
    } else {
        for child in field.children.as_mut_slice() {
            load_field_dictionary(child, reader).await?;
        }
        Ok(())
    }
}

/// Load dictionary value array from manifest files.
// TODO: pub(crate)
pub async fn populate_schema_dictionary<'a>(
    schema: &mut Schema,
    reader: &dyn Reader,
) -> Result<()> {
    for field in schema.fields.as_mut_slice() {
        load_field_dictionary(field, reader).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use arrow_schema::DataType;
    use arrow_schema::Field as ArrowField;
    use arrow_schema::Fields as ArrowFields;
    use arrow_schema::Schema as ArrowSchema;
    use lance_core::datatypes::Schema;

    use crate::datatypes::Fields;
    use crate::datatypes::FieldsWithMeta;

    #[test]
    fn test_schema_set_ids() {
        let arrow_schema = ArrowSchema::new(vec![
            ArrowField::new("a", DataType::Int32, false),
            ArrowField::new(
                "b",
                DataType::Struct(ArrowFields::from(vec![
                    ArrowField::new("f1", DataType::Utf8, true),
                    ArrowField::new("f2", DataType::Boolean, false),
                    ArrowField::new("f3", DataType::Float32, false),
                ])),
                true,
            ),
            ArrowField::new("c", DataType::Float64, false),
        ]);
        let schema = Schema::try_from(&arrow_schema).unwrap();

        let protos: Fields = (&schema).into();
        assert_eq!(
            protos.0.iter().map(|p| p.id).collect::<Vec<_>>(),
            (0..6).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_schema_metadata() {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert(String::from("k1"), String::from("v1"));
        metadata.insert(String::from("k2"), String::from("v2"));

        let arrow_schema = ArrowSchema::new_with_metadata(
            vec![ArrowField::new("a", DataType::Int32, false)],
            metadata,
        );

        let expected_schema = Schema::try_from(&arrow_schema).unwrap();
        let fields_with_meta: FieldsWithMeta = (&expected_schema).into();

        let schema = Schema::from(fields_with_meta);
        assert_eq!(expected_schema, schema);
    }
}
