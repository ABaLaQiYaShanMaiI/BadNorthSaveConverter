use crate::records::*;
use serde_json::Value;
use std::collections::HashMap;

pub struct JsonDecoder;

impl JsonDecoder {
    pub fn decode(json: &Value) -> Result<DeserializedRecord, String> {
        let root_id = json["root_id"]
            .as_i64()
            .ok_or("Missing or invalid root_id")? as i32;
        let header_id = json["header_id"]
            .as_i64()
            .ok_or("Missing or invalid header_id")? as i32;

        let class_types = Self::decode_class_types(&json["class_types"])?;
        let records = Self::decode_records(&json["records"], &class_types)?;
        let record_metadata = Self::decode_record_metadata(json.get("record_metadata"))?;
        let record_order = Self::decode_record_order(json.get("record_order"))?;

        Ok(DeserializedRecord {
            root_id,
            header_id,
            records,
            class_types,
            record_metadata,
            record_order,
        })
    }

    fn decode_record_order(json: Option<&Value>) -> Result<Vec<i32>, String> {
        let Some(value) = json else {
            return Ok(Vec::new());
        };
        let Some(arr) = value.as_array() else {
            return Err("record_order must be an array".to_string());
        };

        let mut order = Vec::with_capacity(arr.len());
        for item in arr {
            order.push(item.as_i64().ok_or("record_order item must be i32")? as i32);
        }
        Ok(order)
    }

    fn decode_record_metadata(
        json: Option<&Value>,
    ) -> Result<HashMap<i32, RecordMetadata>, String> {
        let mut metadata = HashMap::new();
        let Some(value) = json else {
            return Ok(metadata);
        };

        let Some(object) = value.as_object() else {
            return Ok(metadata);
        };

        for (id_str, meta_json) in object {
            let id = id_str
                .parse::<i32>()
                .map_err(|_| format!("Invalid metadata record id: {}", id_str))?;
            let original_record_type = meta_json["original_record_type"]
                .as_u64()
                .ok_or_else(|| format!("Missing original_record_type for record {}", id))?
                as u8;
            let start_offset = meta_json["start_offset"]
                .as_u64()
                .ok_or_else(|| format!("Missing start_offset for record {}", id))?
                as usize;
            let end_offset = meta_json["end_offset"]
                .as_u64()
                .ok_or_else(|| format!("Missing end_offset for record {}", id))?
                as usize;
            let original_byte_length = meta_json["original_byte_length"]
                .as_u64()
                .ok_or_else(|| format!("Missing original_byte_length for record {}", id))?
                as usize;
            metadata.insert(
                id,
                RecordMetadata {
                    original_record_type,
                    start_offset,
                    end_offset,
                    original_byte_length,
                },
            );
        }

        Ok(metadata)
    }

    fn decode_class_types(json: &Value) -> Result<Vec<ClassType>, String> {
        let arr = json.as_array().ok_or("class_types must be an array")?;
        let mut types = vec![];

        for item in arr {
            let id = item["id"].as_u64().ok_or("Missing or invalid type id")? as usize;
            let name = item["name"]
                .as_str()
                .ok_or("Missing type name")?
                .to_string();
            let library_id = item["library_id"].as_i64().ok_or("Missing library_id")? as i32;
            let system_class = item["system_class"]
                .as_bool()
                .ok_or("Missing system_class")?;
            let member_names: Vec<String> = item["member_names"]
                .as_array()
                .ok_or("Missing member_names")?
                .iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect();
            let member_types = Self::decode_member_types(&item["member_types"])?;

            // Ensure types vector is large enough
            while types.len() <= id {
                types.push(ClassType {
                    name: String::new(),
                    library_id: 0,
                    system_class: false,
                    member_names: Vec::new(),
                    member_types: Vec::new(),
                });
            }

            types[id] = ClassType {
                name,
                library_id,
                system_class,
                member_names,
                member_types,
            };
        }

        Ok(types)
    }

    fn decode_records(
        json: &Value,
        class_types: &[ClassType],
    ) -> Result<HashMap<i32, Record>, String> {
        let obj = json.as_object().ok_or("records must be an object")?;
        let mut records = HashMap::new();

        for (id_str, record_json) in obj {
            let id: i32 = id_str
                .parse()
                .map_err(|_| format!("Invalid record id: {}", id_str))?;
            let record = Self::decode_record(record_json, class_types)?;
            records.insert(id, record);
        }

        Ok(records)
    }

    fn decode_record(json: &Value, class_types: &[ClassType]) -> Result<Record, String> {
        let record_type = json["type"].as_str().ok_or("Missing record type")?;

        match record_type {
            "BinaryLibrary" => {
                let value = json["value"]
                    .as_str()
                    .ok_or("Missing BinaryLibrary value")?
                    .to_string();
                Ok(Record::BinaryLibrary(value))
            }
            "String" => {
                let value = json["value"]
                    .as_str()
                    .ok_or("Missing String value")?
                    .to_string();
                Ok(Record::String(value))
            }
            "Class" => {
                let class_type_id = json["class_type_id"]
                    .as_u64()
                    .ok_or("Missing class_type_id")? as usize;

                if class_type_id >= class_types.len() {
                    return Err(format!("Invalid class_type_id: {}", class_type_id));
                }

                let members =
                    Self::decode_class_members(json, &class_types[class_type_id].member_types)?;

                Ok(Record::Class(Class {
                    class_type_id,
                    members,
                }))
            }
            "BinaryArray" => {
                let member_type = Self::decode_member_type(&json["member_type"])?;
                let values = json["values"]
                    .as_array()
                    .ok_or("Missing BinaryArray values")?;
                let mut members = Vec::new();

                for val_json in values {
                    members.push(Self::decode_member(val_json, &member_type)?);
                }

                Ok(Record::BinaryArray(member_type, members))
            }
            "PrimitiveArray" => {
                let prim_type = Self::decode_primitive_type_name(
                    json["primitive_type"]
                        .as_str()
                        .ok_or("Missing primitive_type")?,
                )?;
                let values = json["values"]
                    .as_array()
                    .ok_or("Missing PrimitiveArray values")?;
                let mut primitives = Vec::new();

                for val_json in values {
                    primitives.push(Self::decode_primitive(val_json)?);
                }

                Ok(Record::PrimitiveArray(prim_type, primitives))
            }
            _ => Err(format!("Unknown record type: {}", record_type)),
        }
    }

    fn decode_class_members(json: &Value, types: &[MemberType]) -> Result<Vec<Member>, String> {
        if let Some(members) = json.get("members") {
            return Self::decode_members(members, types);
        }
        Err("Missing class members".to_string())
    }

    fn decode_members(json: &Value, types: &[MemberType]) -> Result<Vec<Member>, String> {
        let arr = json.as_array().ok_or("members must be an array")?;
        if arr.len() != types.len() {
            return Err(format!(
                "members count mismatch: expected {}, got {}",
                types.len(),
                arr.len()
            ));
        }
        let mut members = Vec::new();

        for (member_json, member_type) in arr.iter().zip(types.iter()) {
            members.push(Self::decode_member(member_json, member_type)?);
        }

        Ok(members)
    }

    fn decode_member(json: &Value, _member_type: &MemberType) -> Result<Member, String> {
        let member_type_str = json["type"].as_str().ok_or("Missing member type")?;

        match member_type_str {
            "Reference" => {
                let id = json["id"].as_i64().ok_or("Missing reference id")? as i32;
                Ok(Member::Reference(id))
            }
            "Null" => Ok(Member::Null),
            "NullMultiple" => {
                let count = json["count"].as_i64().ok_or("Missing null count")? as i32;
                Ok(Member::NullMultiple(count))
            }
            "Boolean" => {
                let val = json["value"].as_bool().ok_or("Missing boolean value")?;
                Ok(Member::Primitive(Primitive::Boolean(val)))
            }
            "Byte" => {
                let val = json["value"].as_u64().ok_or("Missing byte value")? as u8;
                Ok(Member::Primitive(Primitive::Byte(val)))
            }
            "Char" => {
                let s = json["value"].as_str().ok_or("Missing char value")?;
                let c = s.chars().next().ok_or("Invalid char")?;
                Ok(Member::Primitive(Primitive::Char(c)))
            }
            "Decimal" => {
                let val = json["value"]
                    .as_str()
                    .ok_or("Missing decimal value")?
                    .to_string();
                Ok(Member::Primitive(Primitive::Decimal(val)))
            }
            "Double" => {
                let val = json["value"].as_f64().ok_or("Missing double value")?;
                Ok(Member::Primitive(Primitive::Double(val)))
            }
            "Int16" => {
                let val = json["value"].as_i64().ok_or("Missing int16 value")? as i16;
                Ok(Member::Primitive(Primitive::Int16(val)))
            }
            "Int32" => {
                let val = json["value"].as_i64().ok_or("Missing int32 value")? as i32;
                Ok(Member::Primitive(Primitive::Int32(val)))
            }
            "Int64" => {
                let val = json["value"].as_i64().ok_or("Missing int64 value")?;
                Ok(Member::Primitive(Primitive::Int64(val)))
            }
            "Int8" => {
                let val = json["value"].as_i64().ok_or("Missing int8 value")? as i8;
                Ok(Member::Primitive(Primitive::Int8(val)))
            }
            "Single" => {
                let val = json["value"].as_f64().ok_or("Missing single value")? as f32;
                Ok(Member::Primitive(Primitive::Single(val)))
            }
            "TimeSpan" => {
                let val = json["value"].as_i64().ok_or("Missing timespan value")?;
                Ok(Member::Primitive(Primitive::TimeSpan(val)))
            }
            "DateTime" => {
                let val = json["value"].as_i64().ok_or("Missing datetime value")?;
                Ok(Member::Primitive(Primitive::DateTime(val)))
            }
            "UInt16" => {
                let val = json["value"].as_u64().ok_or("Missing uint16 value")? as u16;
                Ok(Member::Primitive(Primitive::UInt16(val)))
            }
            "UInt32" => {
                let val = json["value"].as_u64().ok_or("Missing uint32 value")? as u32;
                Ok(Member::Primitive(Primitive::UInt32(val)))
            }
            "UInt64" => {
                let val = json["value"].as_u64().ok_or("Missing uint64 value")?;
                Ok(Member::Primitive(Primitive::UInt64(val)))
            }
            "String" => {
                let val = json["value"]
                    .as_str()
                    .ok_or("Missing string value")?
                    .to_string();
                Ok(Member::Primitive(Primitive::String(val)))
            }
            _ => Err(format!("Unknown member type: {}", member_type_str)),
        }
    }

    fn decode_primitive(json: &Value) -> Result<Primitive, String> {
        let prim_type = json["type"].as_str().ok_or("Missing primitive type")?;

        match prim_type {
            "Boolean" => {
                let val = json["value"].as_bool().ok_or("Missing boolean value")?;
                Ok(Primitive::Boolean(val))
            }
            "Byte" => {
                let val = json["value"].as_u64().ok_or("Missing byte value")? as u8;
                Ok(Primitive::Byte(val))
            }
            "Char" => {
                let s = json["value"].as_str().ok_or("Missing char value")?;
                let c = s.chars().next().ok_or("Invalid char")?;
                Ok(Primitive::Char(c))
            }
            "Decimal" => {
                let val = json["value"]
                    .as_str()
                    .ok_or("Missing decimal value")?
                    .to_string();
                Ok(Primitive::Decimal(val))
            }
            "Double" => {
                let val = json["value"].as_f64().ok_or("Missing double value")?;
                Ok(Primitive::Double(val))
            }
            "Int16" => {
                let val = json["value"].as_i64().ok_or("Missing int16 value")? as i16;
                Ok(Primitive::Int16(val))
            }
            "Int32" => {
                let val = json["value"].as_i64().ok_or("Missing int32 value")? as i32;
                Ok(Primitive::Int32(val))
            }
            "Int64" => {
                let val = json["value"].as_i64().ok_or("Missing int64 value")?;
                Ok(Primitive::Int64(val))
            }
            "Int8" => {
                let val = json["value"].as_i64().ok_or("Missing int8 value")? as i8;
                Ok(Primitive::Int8(val))
            }
            "Single" => {
                let val = json["value"].as_f64().ok_or("Missing single value")? as f32;
                Ok(Primitive::Single(val))
            }
            "TimeSpan" => {
                let val = json["value"].as_i64().ok_or("Missing timespan value")?;
                Ok(Primitive::TimeSpan(val))
            }
            "DateTime" => {
                let val = json["value"].as_i64().ok_or("Missing datetime value")?;
                Ok(Primitive::DateTime(val))
            }
            "UInt16" => {
                let val = json["value"].as_u64().ok_or("Missing uint16 value")? as u16;
                Ok(Primitive::UInt16(val))
            }
            "UInt32" => {
                let val = json["value"].as_u64().ok_or("Missing uint32 value")? as u32;
                Ok(Primitive::UInt32(val))
            }
            "UInt64" => {
                let val = json["value"].as_u64().ok_or("Missing uint64 value")?;
                Ok(Primitive::UInt64(val))
            }
            "String" => {
                let val = json["value"]
                    .as_str()
                    .ok_or("Missing string value")?
                    .to_string();
                Ok(Primitive::String(val))
            }
            "Null" => Ok(Primitive::Null),
            _ => Err(format!("Unknown primitive type: {}", prim_type)),
        }
    }

    fn decode_member_types(json: &Value) -> Result<Vec<MemberType>, String> {
        let arr = json.as_array().ok_or("member_types must be an array")?;
        let mut types = Vec::new();

        for type_json in arr {
            types.push(Self::decode_member_type(type_json)?);
        }

        Ok(types)
    }

    fn decode_member_type(json: &Value) -> Result<MemberType, String> {
        let type_str = json["type"].as_str().ok_or("Missing member type")?;

        match type_str {
            "String" => Ok(MemberType::String),
            "Object" => Ok(MemberType::Object),
            "ObjectArray" => Ok(MemberType::ObjectArray),
            "StringArray" => Ok(MemberType::StringArray),
            "Primitive" => {
                let prim_type_str = json["primitive_type"]
                    .as_str()
                    .ok_or("Missing primitive_type")?;
                let prim_type = Self::decode_primitive_type_name(prim_type_str)?;
                Ok(MemberType::Primitive(prim_type))
            }
            "SystemClass" => {
                let name = json["name"]
                    .as_str()
                    .ok_or("Missing SystemClass name")?
                    .to_string();
                Ok(MemberType::SystemClass(name))
            }
            "Class" => {
                let name = json["name"]
                    .as_str()
                    .ok_or("Missing Class name")?
                    .to_string();
                let library_id = json["library_id"].as_i64().ok_or("Missing library_id")? as i32;
                Ok(MemberType::Class(name, library_id))
            }
            "PrimitiveArray" => {
                let prim_type_str = json["primitive_type"]
                    .as_str()
                    .ok_or("Missing primitive_type")?;
                let prim_type = Self::decode_primitive_type_name(prim_type_str)?;
                Ok(MemberType::PrimitiveArray(prim_type))
            }
            _ => Err(format!("Unknown member type: {}", type_str)),
        }
    }

    fn decode_primitive_type_name(name: &str) -> Result<PrimitiveType, String> {
        Ok(match name {
            "Boolean" => PrimitiveType::Boolean,
            "Byte" => PrimitiveType::Byte,
            "Char" => PrimitiveType::Char,
            "Decimal" => PrimitiveType::Decimal,
            "Double" => PrimitiveType::Double,
            "Int16" => PrimitiveType::Int16,
            "Int32" => PrimitiveType::Int32,
            "Int64" => PrimitiveType::Int64,
            "Int8" => PrimitiveType::Int8,
            "Single" => PrimitiveType::Single,
            "TimeSpan" => PrimitiveType::TimeSpan,
            "DateTime" => PrimitiveType::DateTime,
            "UInt16" => PrimitiveType::UInt16,
            "UInt32" => PrimitiveType::UInt32,
            "UInt64" => PrimitiveType::UInt64,
            "Null" => PrimitiveType::Null,
            "String" => PrimitiveType::String,
            _ => return Err(format!("Unknown primitive type: {}", name)),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::JsonDecoder;

    #[test]
    fn decode_rejects_class_member_count_mismatch() {
        let value = json!({
            "root_id": 1,
            "header_id": -1,
            "class_types": [{
                "id": 0,
                "name": "Root",
                "library_id": 2,
                "system_class": false,
                "member_names": ["a", "b"],
                "member_types": [
                    {"type":"Primitive","primitive_type":"Int32"},
                    {"type":"Object"}
                ]
            }],
            "records": {
                "1": {
                    "type": "Class",
                    "class_type_id": 0,
                    "members": [
                        {"type":"Int32","value":1}
                    ]
                }
            }
        });

        let error = JsonDecoder::decode(&value).expect_err("should fail");
        assert!(error.contains("members count mismatch: expected 2, got 1"));
    }
}
