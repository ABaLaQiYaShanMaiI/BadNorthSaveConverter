use std::collections::HashMap;
use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use super::records::*;

pub fn parse(bytes: &[u8]) -> Result<DeserializedRecord, String> {
    Parser::new(bytes).parse()
}

struct Parser<'a> {
    input: Cursor<&'a [u8]>,
    class_metadata: HashMap<i32, usize>,
    class_types: Vec<ClassType>,
    records: HashMap<i32, Record>,
    record_metadata: HashMap<i32, RecordMetadata>,
    record_order: Vec<i32>,
}

impl<'a> Parser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            input: Cursor::new(bytes),
            class_metadata: HashMap::new(),
            class_types: Vec::new(),
            records: HashMap::new(),
            record_metadata: HashMap::new(),
            record_order: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<DeserializedRecord, String> {
        let header_record_type = self.read_u8()?;
        if header_record_type != 0 {
            return Err(format!(
                "Expected SerializationHeaderRecord (0), got {}",
                header_record_type
            ));
        }

        let root_id = self.read_i32()?;
        let header_id = self.read_i32()?;
        let _major_version = self.read_i32()?;
        let _minor_version = self.read_i32()?;

        loop {
            let record_start = self.input.position() as usize;
            let record_type = self.read_u8()?;
            if record_type == 11 {
                break;
            }

            let record_id = self.read_record(record_type)?;
            self.capture_record_metadata(record_id, record_type, record_start);
        }

        Ok(DeserializedRecord {
            root_id,
            header_id,
            records: self.records,
            class_types: self.class_types,
            record_metadata: self.record_metadata,
            record_order: self.record_order,
        })
    }

    fn capture_record_metadata(&mut self, record_id: i32, record_type: u8, start_offset: usize) {
        let end_offset = self.input.position() as usize;
        self.record_metadata.insert(
            record_id,
            RecordMetadata {
                original_record_type: record_type,
                start_offset,
                end_offset,
                original_byte_length: end_offset.saturating_sub(start_offset),
            },
        );
        self.record_order.push(record_id);
    }

    fn read_record(&mut self, record_type: u8) -> Result<i32, String> {
        match record_type {
            1 => self.read_class_with_id(),
            4 => self.read_class_with_members_and_types(true),
            5 => self.read_class_with_members_and_types(false),
            6 => self.read_binary_object_string(),
            7 => self.read_binary_array(),
            12 => self.read_binary_library(),
            15 => self.read_array_single_primitive(),
            _ => Err(format!("Unsupported record type: {}", record_type)),
        }
    }

    fn read_class_with_members_and_types(&mut self, system_class: bool) -> Result<i32, String> {
        let (id, mut class_type) = self.read_class_type()?;
        class_type.system_class = system_class;
        if !system_class {
            class_type.library_id = self.read_i32()?;
        }

        let class_type_id = self.class_types.len();
        self.class_types.push(class_type);
        self.class_metadata.insert(id, class_type_id);

        let member_types = self.class_types[class_type_id].member_types.clone();
        let members = self.read_members(&member_types)?;

        self.records.insert(
            id,
            Record::Class(Class {
                class_type_id,
                members,
            }),
        );

        Ok(id)
    }

    fn read_class_with_id(&mut self) -> Result<i32, String> {
        let object_id = self.read_i32()?;
        let metadata_id = self.read_i32()?;
        let class_type_id = *self
            .class_metadata
            .get(&metadata_id)
            .ok_or_else(|| format!("Unknown class metadata id {}", metadata_id))?;

        let member_types = self.class_types[class_type_id].member_types.clone();
        let members = self.read_members(&member_types)?;

        self.records.insert(
            object_id,
            Record::Class(Class {
                class_type_id,
                members,
            }),
        );

        Ok(object_id)
    }

    fn read_class_type(&mut self) -> Result<(i32, ClassType), String> {
        let id = self.read_i32()?;
        let name = self.read_string()?;
        let member_count = self.read_i32()?;
        if member_count < 0 {
            return Err("Negative member count in class type".to_string());
        }

        let member_count = member_count as usize;
        let mut member_names = Vec::with_capacity(member_count);
        for _ in 0..member_count {
            member_names.push(self.read_string()?);
        }

        let mut member_types = Vec::with_capacity(member_count);
        for _ in 0..member_count {
            member_types.push(self.read_member_type()?);
        }

        for typ in &mut member_types {
            self.read_member_type_additional_info(typ)?;
        }

        Ok((
            id,
            ClassType {
                name,
                library_id: 0,
                system_class: false,
                member_names,
                member_types,
            },
        ))
    }

    fn read_members(&mut self, member_types: &[MemberType]) -> Result<Vec<Member>, String> {
        let mut members = Vec::with_capacity(member_types.len());
        for member_type in member_types {
            members.push(self.read_member(member_type)?);
        }
        Ok(members)
    }

    fn read_member(&mut self, expected: &MemberType) -> Result<Member, String> {
        if let MemberType::Primitive(primitive_type) = expected {
            return Ok(Member::Primitive(self.read_primitive(*primitive_type)?));
        }

        let binary_type = self.read_u8()?;
        let nested_record_start = (self.input.position() as usize).saturating_sub(1);
        match binary_type {
            1 => {
                let id = self.read_class_with_id()?;
                self.capture_record_metadata(id, binary_type, nested_record_start);
                Ok(Member::Reference(id))
            }
            4 => {
                let id = self.read_class_with_members_and_types(true)?;
                self.capture_record_metadata(id, binary_type, nested_record_start);
                Ok(Member::Reference(id))
            }
            5 => {
                let id = self.read_class_with_members_and_types(false)?;
                self.capture_record_metadata(id, binary_type, nested_record_start);
                Ok(Member::Reference(id))
            }
            6 => {
                let id = self.read_binary_object_string()?;
                self.capture_record_metadata(id, binary_type, nested_record_start);
                Ok(Member::Reference(id))
            }
            7 => {
                let id = self.read_binary_array()?;
                self.capture_record_metadata(id, binary_type, nested_record_start);
                Ok(Member::Reference(id))
            }
            8 => {
                let primitive_type = self.read_primitive_type()?;
                Ok(Member::Primitive(self.read_primitive(primitive_type)?))
            }
            9 => Ok(Member::Reference(self.read_i32()?)),
            10 => Ok(Member::Null),
            13 => Ok(Member::NullMultiple(self.read_u8()? as i32)),
            14 => Ok(Member::NullMultiple(self.read_i32()?)),
            15 => {
                let id = self.read_array_single_primitive()?;
                self.capture_record_metadata(id, binary_type, nested_record_start);
                Ok(Member::Reference(id))
            }
            _ => Err(format!("Unsupported member record type: {}", binary_type)),
        }
    }

    fn read_binary_object_string(&mut self) -> Result<i32, String> {
        let id = self.read_i32()?;
        let value = self.read_string()?;
        self.records.insert(id, Record::String(value));
        Ok(id)
    }

    fn read_binary_array(&mut self) -> Result<i32, String> {
        let id = self.read_i32()?;
        let array_type = self.read_u8()?;
        if array_type != 0 {
            return Err(format!("Unsupported binary array type: {}", array_type));
        }

        let rank = self.read_i32()?;
        if rank != 1 {
            return Err(format!("Unsupported binary array rank: {}", rank));
        }

        let length = self.read_i32()?;
        if length < 0 {
            return Err("Negative binary array length".to_string());
        }

        let mut member_type = self.read_member_type()?;
        self.read_member_type_additional_info(&mut member_type)?;

        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            values.push(self.read_member(&member_type)?);
        }

        self.records
            .insert(id, Record::BinaryArray(member_type, values));

        Ok(id)
    }

    fn read_array_single_primitive(&mut self) -> Result<i32, String> {
        let id = self.read_i32()?;
        let length = self.read_i32()?;
        if length < 0 {
            return Err("Negative primitive array length".to_string());
        }

        let primitive_type = self.read_primitive_type()?;
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            values.push(self.read_primitive(primitive_type)?);
        }

        self.records
            .insert(id, Record::PrimitiveArray(primitive_type, values));

        Ok(id)
    }

    fn read_binary_library(&mut self) -> Result<i32, String> {
        let id = self.read_i32()?;
        let name = self.read_string()?;
        self.records.insert(id, Record::BinaryLibrary(name));
        Ok(id)
    }

    fn read_member_type(&mut self) -> Result<MemberType, String> {
        match self.read_u8()? {
            0 => Ok(MemberType::Primitive(PrimitiveType::Null)),
            1 => Ok(MemberType::String),
            2 => Ok(MemberType::Object),
            3 => Ok(MemberType::SystemClass(String::new())),
            4 => Ok(MemberType::Class(String::new(), 0)),
            5 => Ok(MemberType::ObjectArray),
            6 => Ok(MemberType::StringArray),
            7 => Ok(MemberType::PrimitiveArray(PrimitiveType::Null)),
            v => Err(format!("Unknown member type code: {}", v)),
        }
    }

    fn read_member_type_additional_info(&mut self, typ: &mut MemberType) -> Result<(), String> {
        match typ {
            MemberType::Primitive(t) => *t = self.read_primitive_type()?,
            MemberType::SystemClass(name) => *name = self.read_string()?,
            MemberType::Class(name, library_id) => {
                *name = self.read_string()?;
                *library_id = self.read_i32()?;
            }
            MemberType::PrimitiveArray(t) => *t = self.read_primitive_type()?,
            _ => {}
        }

        Ok(())
    }

    fn read_primitive_type(&mut self) -> Result<PrimitiveType, String> {
        Ok(match self.read_u8()? {
            1 => PrimitiveType::Boolean,
            2 => PrimitiveType::Byte,
            3 => PrimitiveType::Char,
            5 => PrimitiveType::Decimal,
            6 => PrimitiveType::Double,
            7 => PrimitiveType::Int16,
            8 => PrimitiveType::Int32,
            9 => PrimitiveType::Int64,
            10 => PrimitiveType::Int8,
            11 => PrimitiveType::Single,
            12 => PrimitiveType::TimeSpan,
            13 => PrimitiveType::DateTime,
            14 => PrimitiveType::UInt16,
            15 => PrimitiveType::UInt32,
            16 => PrimitiveType::UInt64,
            17 => PrimitiveType::Null,
            18 => PrimitiveType::String,
            v => return Err(format!("Unknown primitive type code: {}", v)),
        })
    }

    fn read_primitive(&mut self, typ: PrimitiveType) -> Result<Primitive, String> {
        Ok(match typ {
            PrimitiveType::Boolean => Primitive::Boolean(self.read_u8()? != 0),
            PrimitiveType::Byte => Primitive::Byte(self.read_u8()?),
            PrimitiveType::Char => {
                let value = self.read_u16()? as u32;
                let character = char::from_u32(value)
                    .ok_or_else(|| format!("Invalid UTF-16 char value: {}", value))?;
                Primitive::Char(character)
            }
            PrimitiveType::Decimal => Primitive::Decimal(self.read_string()?),
            PrimitiveType::Double => Primitive::Double(self.read_f64()?),
            PrimitiveType::Int16 => Primitive::Int16(self.read_i16()?),
            PrimitiveType::Int32 => Primitive::Int32(self.read_i32()?),
            PrimitiveType::Int64 => Primitive::Int64(self.read_i64()?),
            PrimitiveType::Int8 => Primitive::Int8(self.read_i8()?),
            PrimitiveType::Single => Primitive::Single(self.read_f32()?),
            PrimitiveType::TimeSpan => Primitive::TimeSpan(self.read_i64()?),
            PrimitiveType::DateTime => Primitive::DateTime(self.read_i64()?),
            PrimitiveType::UInt16 => Primitive::UInt16(self.read_u16()?),
            PrimitiveType::UInt32 => Primitive::UInt32(self.read_u32()?),
            PrimitiveType::UInt64 => Primitive::UInt64(self.read_u64()?),
            PrimitiveType::Null => Primitive::Null,
            PrimitiveType::String => Primitive::String(self.read_string()?),
        })
    }

    fn read_string(&mut self) -> Result<String, String> {
        let length = self.read_7bit_encoded_int()?;
        let mut bytes = vec![0u8; length];
        self.input
            .read_exact(&mut bytes)
            .map_err(|e| format!("Failed to read string bytes: {}", e))?;
        String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 string: {}", e))
    }

    fn read_7bit_encoded_int(&mut self) -> Result<usize, String> {
        let mut result = 0usize;
        let mut shift = 0usize;

        for _ in 0..5 {
            let byte = self.read_u8()? as usize;
            result |= (byte & 0x7F) << shift;
            if (byte & 0x80) == 0 {
                return Ok(result);
            }
            shift += 7;
        }

        Err("Invalid 7-bit encoded int (too long)".to_string())
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        self.input
            .read_u8()
            .map_err(|e| format!("Failed to read u8: {}", e))
    }

    fn read_i8(&mut self) -> Result<i8, String> {
        self.input
            .read_i8()
            .map_err(|e| format!("Failed to read i8: {}", e))
    }

    fn read_i16(&mut self) -> Result<i16, String> {
        self.input
            .read_i16::<LittleEndian>()
            .map_err(|e| format!("Failed to read i16: {}", e))
    }

    fn read_u16(&mut self) -> Result<u16, String> {
        self.input
            .read_u16::<LittleEndian>()
            .map_err(|e| format!("Failed to read u16: {}", e))
    }

    fn read_i32(&mut self) -> Result<i32, String> {
        self.input
            .read_i32::<LittleEndian>()
            .map_err(|e| format!("Failed to read i32: {}", e))
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        self.input
            .read_u32::<LittleEndian>()
            .map_err(|e| format!("Failed to read u32: {}", e))
    }

    fn read_i64(&mut self) -> Result<i64, String> {
        self.input
            .read_i64::<LittleEndian>()
            .map_err(|e| format!("Failed to read i64: {}", e))
    }

    fn read_u64(&mut self) -> Result<u64, String> {
        self.input
            .read_u64::<LittleEndian>()
            .map_err(|e| format!("Failed to read u64: {}", e))
    }

    fn read_f32(&mut self) -> Result<f32, String> {
        self.input
            .read_f32::<LittleEndian>()
            .map_err(|e| format!("Failed to read f32: {}", e))
    }

    fn read_f64(&mut self) -> Result<f64, String> {
        self.input
            .read_f64::<LittleEndian>()
            .map_err(|e| format!("Failed to read f64: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::parse;
    use crate::records::{
        Class, ClassType, DeserializedRecord, Member, MemberType, Primitive, PrimitiveType, Record,
        RecordMetadata,
    };
    use crate::serializer::{serialize, serialize_checked};

    #[test]
    fn parse_serialize_roundtrip() {
        let mut records = HashMap::new();

        records.insert(2, Record::BinaryLibrary("Assembly-CSharp".to_string()));
        records.insert(3, Record::String("hello".to_string()));
        records.insert(
            1,
            Record::Class(Class {
                class_type_id: 0,
                members: vec![
                    Member::Primitive(Primitive::Int32(19)),
                    Member::Reference(3),
                    Member::Primitive(Primitive::Boolean(true)),
                ],
            }),
        );

        let class_types = vec![ClassType {
            name: "CampaignSave".to_string(),
            library_id: 2,
            system_class: false,
            member_names: vec![
                "serializedVersion".to_string(),
                "name".to_string(),
                "enabled".to_string(),
            ],
            member_types: vec![
                MemberType::Primitive(PrimitiveType::Int32),
                MemberType::String,
                MemberType::Primitive(PrimitiveType::Boolean),
            ],
        }];

        let source = DeserializedRecord {
            root_id: 1,
            header_id: -1,
            records,
            class_types,
            record_metadata: HashMap::new(),
            record_order: Vec::new(),
        };

        let bytes = serialize(&source);
        let parsed = parse(&bytes).expect("roundtrip parse should succeed");

        assert_eq!(parsed.root_id, source.root_id);
        assert_eq!(parsed.header_id, source.header_id);
        assert_eq!(parsed.class_types, source.class_types);
        assert_eq!(parsed.records, source.records);
        assert!(parsed.record_metadata.contains_key(&1));
        assert!(parsed.record_metadata.contains_key(&2));
        assert!(parsed.record_metadata.contains_key(&3));
    }

    #[test]
    fn parse_member_inline_class_with_members_and_types() {
        // 内联记录类型5测试
        let mut bytes = Vec::new();
        write_u8(&mut bytes, 0);
        write_i32(&mut bytes, 1);
        write_i32(&mut bytes, -1);
        write_i32(&mut bytes, 1);
        write_i32(&mut bytes, 0);

        write_u8(&mut bytes, 5);
        write_i32(&mut bytes, 1);
        write_string(&mut bytes, "Root");
        write_i32(&mut bytes, 1);
        write_string(&mut bytes, "child");
        write_u8(&mut bytes, 2);
        write_i32(&mut bytes, 2);
        write_u8(&mut bytes, 5);

        write_i32(&mut bytes, 3);
        write_string(&mut bytes, "Child");
        write_i32(&mut bytes, 1);
        write_string(&mut bytes, "value");
        write_u8(&mut bytes, 0);
        write_u8(&mut bytes, 8);
        write_i32(&mut bytes, 2);
        write_i32(&mut bytes, 42);

        write_u8(&mut bytes, 11);

        let parsed = parse(&bytes).expect("parse should support inline record type 5 in member");
        let root = parsed.records[&1].as_class();
        assert_eq!(root.members[0], Member::Reference(3));

        let child = parsed.records[&3].as_class();
        assert_eq!(child.members[0], Member::Primitive(Primitive::Int32(42)));
        assert_eq!(parsed.record_metadata[&1].original_record_type, 5);
        assert_eq!(parsed.record_metadata[&3].original_record_type, 5);
    }

    #[test]
    fn parse_member_inline_array_single_primitive() {
        let mut bytes = Vec::new();
        write_u8(&mut bytes, 0);
        write_i32(&mut bytes, 1);
        write_i32(&mut bytes, -1);
        write_i32(&mut bytes, 1);
        write_i32(&mut bytes, 0);

        write_u8(&mut bytes, 5);
        write_i32(&mut bytes, 1);
        write_string(&mut bytes, "Root");
        write_i32(&mut bytes, 1);
        write_string(&mut bytes, "arr");
        write_u8(&mut bytes, 2);
        write_i32(&mut bytes, 2);
        write_u8(&mut bytes, 15);
        write_i32(&mut bytes, 4);
        write_i32(&mut bytes, 2);
        write_u8(&mut bytes, 8);
        write_i32(&mut bytes, 10);
        write_i32(&mut bytes, 20);

        write_u8(&mut bytes, 11);

        let parsed = parse(&bytes).expect("parse should support inline record type 15 in member");
        let root = parsed.records[&1].as_class();
        assert_eq!(root.members[0], Member::Reference(4));
        assert_eq!(
            parsed.records[&4],
            Record::PrimitiveArray(
                PrimitiveType::Int32,
                vec![Primitive::Int32(10), Primitive::Int32(20)]
            )
        );
        assert_eq!(parsed.record_metadata[&1].original_record_type, 5);
        assert_eq!(parsed.record_metadata[&4].original_record_type, 15);
    }

    #[test]
    fn parse_serialize_checked_preserves_inline_string_record_bytes() {
        let mut bytes = Vec::new();
        write_u8(&mut bytes, 0);
        write_i32(&mut bytes, 1);
        write_i32(&mut bytes, -1);
        write_i32(&mut bytes, 1);
        write_i32(&mut bytes, 0);

        write_u8(&mut bytes, 5);
        write_i32(&mut bytes, 1);
        write_string(&mut bytes, "Root");
        write_i32(&mut bytes, 1);
        write_string(&mut bytes, "name");
        write_u8(&mut bytes, 1);
        write_i32(&mut bytes, 2);
        write_u8(&mut bytes, 6);
        write_i32(&mut bytes, 3);
        write_string(&mut bytes, "hello");

        write_u8(&mut bytes, 11);

        let parsed = parse(&bytes).expect("parse should succeed");
        let serialized = serialize_checked(&parsed).expect("serialize should succeed");
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn serialize_checked_reports_record_size_mismatch() {
        // PrimitiveArray (type 15) is still strictly size-validated; use it to verify
        // that the size-mismatch error message includes the expected diagnostics.
        let mut records = HashMap::new();
        records.insert(
            2,
            Record::PrimitiveArray(
                PrimitiveType::Int32,
                vec![Primitive::Int32(1), Primitive::Int32(2)],
            ),
        );
        let source = DeserializedRecord {
            root_id: 2,
            header_id: -1,
            records,
            class_types: Vec::new(),
            record_metadata: HashMap::from([(
                2,
                RecordMetadata {
                    original_record_type: 15,
                    start_offset: 100,
                    end_offset: 105,
                    original_byte_length: 1,
                },
            )]),
            record_order: Vec::new(),
        };

        let error = serialize_checked(&source).expect_err("should reject mismatched size");
        assert!(error.contains("record_id=2"));
        assert!(error.contains("original offsets 100..105"));
    }

    #[test]
    fn serialize_checked_accepts_string_record_size_difference() {
        // String records (type 6) use a self-describing 7-bit length prefix, so a change
        // in the string value length must be accepted by the serializer.
        let mut records = HashMap::new();
        records.insert(2, Record::String("hello".to_string()));
        let source = DeserializedRecord {
            root_id: 2,
            header_id: -1,
            records,
            class_types: Vec::new(),
            record_metadata: HashMap::from([(
                2,
                RecordMetadata {
                    original_record_type: 6,
                    start_offset: 100,
                    end_offset: 105,
                    original_byte_length: 1,
                },
            )]),
            record_order: Vec::new(),
        };

        serialize_checked(&source).expect("string record size difference should be accepted");
    }

    fn write_u8(bytes: &mut Vec<u8>, value: u8) {
        bytes.push(value);
    }

    fn write_i32(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn write_string(bytes: &mut Vec<u8>, value: &str) {
        write_7bit_encoded_int(bytes, value.len() as u32);
        bytes.extend_from_slice(value.as_bytes());
    }

    fn write_7bit_encoded_int(bytes: &mut Vec<u8>, mut value: u32) {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }
}
