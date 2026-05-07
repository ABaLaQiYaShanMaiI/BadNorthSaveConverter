use std::collections::{HashMap, HashSet, VecDeque};

use byteorder::{LittleEndian, WriteBytesExt};

use super::records::*;

pub fn serialize(rec: &DeserializedRecord) -> Vec<u8> {
    serialize_checked(rec).unwrap_or_else(|error| panic!("{}", error))
}

pub fn serialize_checked(rec: &DeserializedRecord) -> Result<Vec<u8>, String> {
    rec.validate_graph()?;
    Serializer::new().serialize(rec)
}

struct Serializer {
    output: Vec<u8>,
    todo: VecDeque<i32>,
    done: HashSet<i32>,
    emitted: HashSet<i32>,
    class_metadata: HashMap<usize, i32>,
}

impl Serializer {
    fn new() -> Self {
        Self {
            output: Vec::with_capacity(0x1000),
            todo: VecDeque::new(),
            done: HashSet::new(),
            emitted: HashSet::new(),
            class_metadata: HashMap::new(),
        }
    }

    fn add_todo(&mut self, id: i32) {
        if !self.done.contains(&id) {
            self.done.insert(id);
            self.todo.push_back(id);
        }
    }

    fn serialize(mut self, rec: &DeserializedRecord) -> Result<Vec<u8>, String> {
        self.write_u8(0);
        self.write_i32(rec.root_id);
        self.write_i32(rec.header_id);
        self.write_i32(1);
        self.write_i32(0);

        if let Some(original_top_level_order) = self.original_top_level_order(rec) {
            for id in original_top_level_order {
                self.write_record_by_id(rec, id)?;
            }
        } else {
            let mut library_ids: Vec<i32> = rec
                .records
                .iter()
                .filter_map(|(id, record)| match record {
                    Record::BinaryLibrary(_) => Some(*id),
                    _ => None,
                })
                .collect();
            library_ids.sort_by_key(|id| {
                rec.record_metadata(*id)
                    .map_or((usize::MAX, *id), |metadata| (metadata.start_offset, *id))
            });
            for id in library_ids {
                self.add_todo(id);
            }
            self.add_todo(rec.root_id);
        }

        while let Some(id) = self.todo.pop_front() {
            self.write_record_by_id(rec, id)?;
        }

        let mut remaining_ids: Vec<i32> = rec
            .records
            .keys()
            .filter(|id| !self.emitted.contains(id))
            .copied()
            .collect();
        remaining_ids.sort_by_key(|id| {
            rec.record_metadata(*id)
                .map_or((usize::MAX, *id), |metadata| (metadata.start_offset, *id))
        });
        for id in remaining_ids {
            self.write_record_by_id(rec, id)?;
        }

        self.write_u8(11);

        Ok(self.output)
    }

    fn write_record_by_id(&mut self, recs: &DeserializedRecord, id: i32) -> Result<(), String> {
        if self.emitted.contains(&id) {
            return Ok(());
        }
        let Some(record) = recs.records.get(&id) else {
            return Err(format!("Missing record {}", id));
        };
        let start_offset = self.output.len();
        self.write_record(recs, id, record)?;
        self.validate_record_size(recs, id, start_offset)?;
        self.emitted.insert(id);
        self.done.insert(id);
        Ok(())
    }

    fn original_top_level_order(&self, recs: &DeserializedRecord) -> Option<Vec<i32>> {
        if recs.record_metadata.is_empty() {
            return None;
        }
        let mut sorted_ids = recs.record_order.clone();
        if sorted_ids.is_empty() {
            sorted_ids = recs.record_metadata.keys().copied().collect();
            sorted_ids.sort_by_key(|id| {
                recs.record_metadata(*id)
                    .map_or((usize::MAX, *id), |metadata| (metadata.start_offset, *id))
            });
        }
        Some(
            sorted_ids
                .into_iter()
                .filter(|id| !self.is_nested_record(recs, *id))
                .collect(),
        )
    }

    fn is_nested_record(&self, recs: &DeserializedRecord, record_id: i32) -> bool {
        let Some(record_meta) = recs.record_metadata(record_id) else {
            return false;
        };
        recs.record_metadata.iter().any(|(other_id, other_meta)| {
            *other_id != record_id
                && other_meta.start_offset < record_meta.start_offset
                && record_meta.end_offset <= other_meta.end_offset
        })
    }

    fn should_inline_member_record(
        &self,
        recs: &DeserializedRecord,
        parent_record_id: Option<i32>,
        child_record_id: i32,
    ) -> bool {
        let Some(parent_id) = parent_record_id else {
            return false;
        };
        let Some(parent_meta) = recs.record_metadata(parent_id) else {
            return false;
        };
        let Some(child_meta) = recs.record_metadata(child_record_id) else {
            return false;
        };

        parent_meta.start_offset < child_meta.start_offset
            && child_meta.end_offset <= parent_meta.end_offset
    }

    fn write_record(
        &mut self,
        recs: &DeserializedRecord,
        id: i32,
        record: &Record,
    ) -> Result<(), String> {
        match record {
            Record::BinaryLibrary(name) => {
                self.ensure_record_type(id, 12, recs)?;
                self.write_u8(12);
                self.write_i32(id);
                self.write_string(name);
            }
            Record::Class(class) => {
                let class_type = recs.class_type(class);
                let record_type = self.resolve_class_record_type(recs, id, class, class_type);

                match record_type {
                    1 => {
                        let class_id = *self.class_metadata.get(&class.class_type_id).ok_or_else(|| {
                            format!(
                                "Record {} requires ClassWithId(1) but no metadata class id is available for class_type_id {}",
                                id, class.class_type_id
                            )
                        })?;
                        self.write_u8(1);
                        self.write_i32(id);
                        self.write_i32(class_id);
                    }
                    4 => {
                        self.write_u8(4);
                        self.write_class_type(id, class_type);
                        self.class_metadata.insert(class.class_type_id, id);
                    }
                    5 => {
                        self.write_u8(5);
                        self.write_class_type(id, class_type);
                        if self.class_record_writes_library_id(record_type) {
                            self.write_i32(class_type.library_id);
                        }
                        self.class_metadata.insert(class.class_type_id, id);
                    }
                    _ => {
                        return Err(format!(
                            "Record {} is a Class but metadata requires unsupported record type {}",
                            id, record_type
                        ));
                    }
                }

                for (member, member_type) in
                    class.members.iter().zip(class_type.member_types.iter())
                {
                    self.write_member(recs, Some(id), member, member_type)?;
                }
            }
            Record::BinaryArray(typ, vals) => {
                self.ensure_record_type(id, 7, recs)?;
                self.write_u8(7);
                self.write_i32(id);
                self.write_u8(0);
                self.write_i32(1);
                self.write_i32(vals.len() as i32);
                self.write_member_type(typ);
                self.write_member_type_additional_info(typ);
                for val in vals {
                    self.write_member(recs, Some(id), val, typ)?;
                }
            }
            Record::PrimitiveArray(typ, vals) => {
                self.ensure_record_type(id, 15, recs)?;
                self.write_u8(15);
                self.write_i32(id);
                self.write_i32(vals.len() as i32);
                self.write_primitive_type(typ);
                for val in vals {
                    self.write_primitive(val);
                }
            }
            Record::String(val) => {
                self.ensure_record_type(id, 6, recs)?;
                self.write_u8(6);
                self.write_i32(id);
                self.write_string(val);
            }
        }

        Ok(())
    }

    fn ensure_record_type(
        &self,
        id: i32,
        writer_record_type: u8,
        recs: &DeserializedRecord,
    ) -> Result<(), String> {
        if let Some(metadata) = recs.record_metadata(id) {
            if metadata.original_record_type != writer_record_type {
                return Err(format!(
                    "Record {} metadata/type mismatch: original record type {} but writer would emit {}",
                    id, metadata.original_record_type, writer_record_type
                ));
            }
        }
        Ok(())
    }

    fn resolve_class_record_type(
        &self,
        recs: &DeserializedRecord,
        id: i32,
        class: &Class,
        class_type: &ClassType,
    ) -> u8 {
        if let Some(value) = recs
            .record_metadata(id)
            .map(|metadata| metadata.original_record_type)
        {
            return value;
        }

        if class_type.system_class {
            4
        } else if self.class_metadata.contains_key(&class.class_type_id) {
            1
        } else {
            5
        }
    }

    fn class_record_writes_library_id(&self, record_type: u8) -> bool {
        record_type == 5
    }

    fn write_class_type(&mut self, id: i32, class_type: &ClassType) {
        self.write_i32(id);
        self.write_string(&class_type.name);
        self.write_i32(class_type.member_names.len() as i32);

        for name in &class_type.member_names {
            self.write_string(name);
        }

        for t in &class_type.member_types {
            self.write_member_type(t);
        }

        for t in &class_type.member_types {
            self.write_member_type_additional_info(t);
        }
    }

    fn write_member_type(&mut self, typ: &MemberType) {
        match typ {
            MemberType::Primitive(_) => self.write_u8(0),
            MemberType::String => self.write_u8(1),
            MemberType::Object => self.write_u8(2),
            MemberType::SystemClass(_) => self.write_u8(3),
            MemberType::Class(..) => self.write_u8(4),
            MemberType::ObjectArray => self.write_u8(5),
            MemberType::StringArray => self.write_u8(6),
            MemberType::PrimitiveArray(_) => self.write_u8(7),
        }
    }

    fn write_member_type_additional_info(&mut self, typ: &MemberType) {
        match typ {
            MemberType::Primitive(t) => self.write_primitive_type(t),
            MemberType::SystemClass(name) => self.write_string(name),
            MemberType::Class(s, i) => {
                self.write_string(s);
                self.write_i32(*i);
            }
            MemberType::PrimitiveArray(t) => self.write_primitive_type(t),
            _ => (),
        }
    }

    fn write_member(
        &mut self,
        recs: &DeserializedRecord,
        parent_record_id: Option<i32>,
        member: &Member,
        t: &MemberType,
    ) -> Result<(), String> {
        if let MemberType::Primitive(_) = t {
            if let Member::Primitive(val) = member {
                self.write_primitive(val);
            } else {
                return Err("Non primitive member for primitive field".to_string());
            }
        } else {
            match member {
                Member::Primitive(val) => {
                    self.write_u8(8);
                    self.write_primitive_type(&val.primitive_type());
                    self.write_primitive(val);
                }
                Member::Reference(id) => {
                    if self.should_inline_member_record(recs, parent_record_id, *id) {
                        self.write_record_by_id(recs, *id)?;
                    } else {
                        self.write_u8(9);
                        self.write_i32(*id);
                        self.add_todo(*id);
                    }
                }
                Member::Null => self.write_u8(10),
                Member::NullMultiple(count) => {
                    if *count <= 0 {
                        return Err(format!(
                            "NullMultiple count must be positive, got {}",
                            count
                        ));
                    }
                    if *count <= 0xFF {
                        self.write_u8(13);
                        self.write_u8(*count as u8);
                    } else {
                        self.write_u8(14);
                        self.write_i32(*count);
                    }
                }
            }
        }
        Ok(())
    }

    fn write_primitive_type(&mut self, typ: &PrimitiveType) {
        self.write_u8(match typ {
            PrimitiveType::Boolean => 1,
            PrimitiveType::Byte => 2,
            PrimitiveType::Char => 3,
            PrimitiveType::Decimal => 5,
            PrimitiveType::Double => 6,
            PrimitiveType::Int16 => 7,
            PrimitiveType::Int32 => 8,
            PrimitiveType::Int64 => 9,
            PrimitiveType::Int8 => 10,
            PrimitiveType::Single => 11,
            PrimitiveType::TimeSpan => 12,
            PrimitiveType::DateTime => 13,
            PrimitiveType::UInt16 => 14,
            PrimitiveType::UInt32 => 15,
            PrimitiveType::UInt64 => 16,
            PrimitiveType::Null => 17,
            PrimitiveType::String => 18,
        });
    }

    fn write_primitive(&mut self, val: &Primitive) {
        match val {
            Primitive::Boolean(val) => self.write_u8(*val as u8),
            Primitive::Byte(val) => self.write_u8(*val),
            Primitive::Char(val) => {
                let code = *val as u32;
                if code > u16::MAX as u32 {
                    self.output.write_u16::<LittleEndian>(b'?' as u16).unwrap();
                } else {
                    self.output.write_u16::<LittleEndian>(code as u16).unwrap();
                }
            }
            Primitive::Decimal(val) => self.write_string(val),
            Primitive::Double(val) => self.output.write_f64::<LittleEndian>(*val).unwrap(),
            Primitive::Int16(val) => self.output.write_i16::<LittleEndian>(*val).unwrap(),
            Primitive::Int32(val) => self.output.write_i32::<LittleEndian>(*val).unwrap(),
            Primitive::Int64(val) => self.output.write_i64::<LittleEndian>(*val).unwrap(),
            Primitive::Int8(val) => self.output.write_i8(*val).unwrap(),
            Primitive::Single(val) => self.output.write_f32::<LittleEndian>(*val).unwrap(),
            Primitive::TimeSpan(val) => self.output.write_i64::<LittleEndian>(*val).unwrap(),
            Primitive::DateTime(val) => self.output.write_i64::<LittleEndian>(*val).unwrap(),
            Primitive::UInt16(val) => self.output.write_u16::<LittleEndian>(*val).unwrap(),
            Primitive::UInt32(val) => self.output.write_u32::<LittleEndian>(*val).unwrap(),
            Primitive::UInt64(val) => self.output.write_u64::<LittleEndian>(*val).unwrap(),
            Primitive::Null => (),
            Primitive::String(val) => self.write_string(val),
        }
    }

    fn write_string(&mut self, val: &str) {
        let bytes = val.as_bytes();
        let mut length = bytes.len();
        assert!(length <= 0x7FFF_FFFF);
        loop {
            let val = (length & 0b111_1111) as u8;
            length >>= 7;
            if length == 0 {
                self.write_u8(val);
                break;
            }
            self.write_u8(val | 0b1000_0000);
        }
        self.output.extend(bytes);
    }

    fn validate_record_size(
        &self,
        recs: &DeserializedRecord,
        id: i32,
        start_offset: usize,
    ) -> Result<(), String> {
        let metadata = match recs.record_metadata(id) {
            Some(m) => m,
            None => return Ok(()),
        };

        let end_offset = self.output.len();
        let serialized_size = end_offset.saturating_sub(start_offset);

        // 字符串/类型大小可变化
        if matches!(metadata.original_record_type, 1 | 4 | 5 | 6 | 7) {
            return Ok(());
        }
        
        if serialized_size != metadata.original_byte_length {
            let mut message = format!(
                "Record size mismatch at record_id={} (type={}, original offsets {}..{}): serialized offsets {}..{}, original_size={} bytes, serialized_size={} bytes.",
                id,
                metadata.original_record_type,
                metadata.start_offset,
                metadata.end_offset,
                start_offset,
                end_offset,
                metadata.original_byte_length,
                serialized_size
            );

            if let Some(Record::String(value)) = recs.records.get(&id) {
                message.push_str(&format!(
                    " String diagnostics: utf8_bytes={}, utf16_code_units={}.",
                    value.len(),
                    value.encode_utf16().count()
                ));
            }

            if id == 1 {
                if let Some(Record::Class(class)) = recs.records.get(&id) {
                    let class_type = recs.class_type(class);
                    message.push_str(&self.record_member_breakdown(
                        class,
                        class_type,
                        serialized_size,
                    )?);
                }
            }

            return Err(message);
        }
        Ok(())
    }

    fn record_member_breakdown(
        &self,
        class: &Class,
        class_type: &ClassType,
        serialized_size: usize,
    ) -> Result<String, String> {
        let mut diagnostics = String::new();
        diagnostics.push_str(&format!(
            " Record 1 member serialization breakdown for '{}':",
            class_type.name
        ));
        diagnostics.push_str(&format!(
            " expected_members={}, actual_members={}.",
            class_type.member_types.len(),
            class.members.len()
        ));

        let max_count = class_type.member_types.len().max(class.members.len());
        let mut member_total = 0usize;
        for index in 0..max_count {
            let name = class_type
                .member_names
                .get(index)
                .map(String::as_str)
                .unwrap_or("<unknown>");
            let member_type = class_type.member_types.get(index);
            let member = class.members.get(index);
            match (member, member_type) {
                (Some(member), Some(member_type)) => {
                    let size = Self::member_serialized_size(member, member_type)?;
                    let start = member_total;
                    member_total += size;
                    diagnostics.push_str(&format!(
                        " [#{index} '{name}' {member_type:?}] bytes={size} range={start}..{member_total} value={member:?};"
                    ));
                }
                (None, Some(member_type)) => diagnostics.push_str(&format!(
                    " [#{index} '{name}' {member_type:?}] MISSING member value;"
                )),
                (Some(member), None) => diagnostics.push_str(&format!(
                    " [#{index} '<extra>' ] EXTRA member value={member:?};"
                )),
                (None, None) => {}
            }
        }

        let class_header_bytes = serialized_size.saturating_sub(member_total);
        diagnostics.push_str(&format!(
            " class_header_bytes={class_header_bytes} members_total_bytes={member_total}.",
        ));
        Ok(diagnostics)
    }

    fn member_serialized_size(member: &Member, member_type: &MemberType) -> Result<usize, String> {
        if let MemberType::Primitive(_) = member_type {
            if let Member::Primitive(value) = member {
                return Ok(Self::primitive_serialized_size(value));
            }
            return Err("Non primitive member for primitive field".to_string());
        }

        Ok(match member {
            Member::Primitive(value) => 2 + Self::primitive_serialized_size(value),
            Member::Reference(_) => 5,
            Member::Null => 1,
            Member::NullMultiple(count) => {
                if *count <= 0 {
                    return Err(format!("NullMultiple count must be positive, got {}", count));
                }
                if *count <= 0xFF { 2 } else { 5 }
            }
        })
    }

    fn primitive_serialized_size(value: &Primitive) -> usize {
        match value {
            Primitive::Boolean(_) | Primitive::Byte(_) | Primitive::Int8(_) => 1,
            Primitive::Char(_) | Primitive::Int16(_) | Primitive::UInt16(_) => 2,
            Primitive::Int32(_) | Primitive::UInt32(_) | Primitive::Single(_) => 4,
            Primitive::Int64(_)
            | Primitive::UInt64(_)
            | Primitive::Double(_)
            | Primitive::TimeSpan(_)
            | Primitive::DateTime(_) => 8,
            Primitive::Decimal(s) | Primitive::String(s) => Self::encoded_string_size(s),
            Primitive::Null => 0,
        }
    }

    fn encoded_string_size(value: &str) -> usize {
        let length = value.len();
        let prefix_len = if length <= 0x7F {
            1
        } else if length <= 0x3FFF {
            2
        } else if length <= 0x1F_FFFF {
            3
        } else if length <= 0x0FFF_FFFF {
            4
        } else {
            5
        };
        prefix_len + length
    }

    fn write_u8(&mut self, i: u8) {
        self.output.write_u8(i).unwrap();
    }

    fn write_i32(&mut self, i: i32) {
        self.output.write_i32::<LittleEndian>(i).unwrap();
    }
}  

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::serialize_checked;
    use crate::records::{
        Class, ClassType, DeserializedRecord, Member, MemberType, Primitive, PrimitiveType, Record,
        RecordMetadata,
    };

    #[test]
    fn serialize_checked_accepts_class_record_size_difference() {
        // Class records (types 1, 4, 5) may contain string-typed members whose byte
        // lengths can change when field values are replaced (e.g. renaming a hero class).
        // The serializer must accept such size differences without error.
        let mut records = HashMap::new();
        records.insert(
            1,
            Record::Class(Class {
                class_type_id: 0,
                members: vec![Member::Primitive(Primitive::Int32(7)), Member::Null],
            }),
        );

        let source = DeserializedRecord {
            root_id: 1,
            header_id: -1,
            records,
            class_types: vec![ClassType {
                name: "Voxels.TowerDefense.ProfileInternals.CampaignSave".to_string(),
                library_id: 2,
                system_class: false,
                member_names: vec!["serializedVersion".to_string(), "heroes".to_string()],
                member_types: vec![
                    MemberType::Primitive(PrimitiveType::Int32),
                    MemberType::Object,
                ],
            }],
            record_metadata: HashMap::from([(
                1,
                RecordMetadata {
                    original_record_type: 5,
                    start_offset: 17,
                    end_offset: 18,
                    original_byte_length: 1,
                },
            )]),
            record_order: Vec::new(),
        };

        serialize_checked(&source)
            .expect("class record size difference should be accepted");
    }

    #[test]
    fn serialize_checked_rejects_non_positive_null_multiple_count() {
        let mut records = HashMap::new();
        records.insert(
            1,
            Record::Class(Class {
                class_type_id: 0,
                members: vec![Member::NullMultiple(0)],
            }),
        );

        let source = DeserializedRecord {
            root_id: 1,
            header_id: -1,
            records,
            class_types: vec![ClassType {
                name: "Root".to_string(),
                library_id: 2,
                system_class: false,
                member_names: vec!["child".to_string()],
                member_types: vec![MemberType::Object],
            }],
            record_metadata: HashMap::new(),
            record_order: Vec::new(),
        };

        let error = serialize_checked(&source).expect_err("should reject invalid NullMultiple");
        assert!(error.contains("NullMultiple count must be positive"));
    }

    #[test]
    fn serialize_string_uses_7bit_length_prefix() {
        let value = "A".repeat(49);
        let source = DeserializedRecord {
            root_id: 1,
            header_id: -1,
            records: HashMap::from([(1, Record::String(value.clone()))]),
            class_types: Vec::new(),
            record_metadata: HashMap::new(),
            record_order: Vec::new(),
        };

        let bytes = serialize_checked(&source).expect("serialize should succeed");
        assert_eq!(bytes[17], 6);
        assert_eq!(&bytes[18..22], &1i32.to_le_bytes());
        assert_eq!(bytes[22], 0x31);
        assert_eq!(&bytes[23..72], value.as_bytes());
    }
}
