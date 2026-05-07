use super::records::*;
use serde_json::{json, Value};

pub struct JsonEncoder {
    rec: DeserializedRecord,
}

impl JsonEncoder {
    pub fn new(rec: DeserializedRecord) -> Self {
        Self { rec }
    }

    pub fn encode(self) -> Value {
        json!({
            "root_id": self.rec.root_id,
            "header_id": self.rec.header_id,
            "class_types": self.encode_class_types(),
            "records": self.encode_records(),
            "record_metadata": self.encode_record_metadata(),
            "record_order": self.rec.record_order,
        })
    }

    fn encode_class_types(&self) -> Vec<Value> {
        let mut types = Vec::new();
        for (idx, class_type) in self.rec.class_types.iter().enumerate() {
            types.push(json!({
                "id": idx,
                "name": class_type.name,
                "library_id": class_type.library_id,
                "system_class": class_type.system_class,
                "member_names": class_type.member_names,
                "member_types": self.encode_member_types(&class_type.member_types),
            }));
        }
        types
    }

    fn encode_records(&self) -> serde_json::Map<String, Value> {
        let mut records = serde_json::Map::new();
        for (id, record) in &self.rec.records {
            records.insert(id.to_string(), self.encode_record(record));
        }
        records
    }

    fn encode_record_metadata(&self) -> serde_json::Map<String, Value> {
        let mut metadata = serde_json::Map::new();
        for (id, item) in &self.rec.record_metadata {
            metadata.insert(
                id.to_string(),
                json!({
                    "original_record_type": item.original_record_type,
                    "start_offset": item.start_offset,
                    "end_offset": item.end_offset,
                    "original_byte_length": item.original_byte_length,
                }),
            );
        }
        metadata
    }

    fn encode_record(&self, record: &Record) -> Value {
        match record {
            Record::BinaryLibrary(name) => json!({
                "type": "BinaryLibrary",
                "value": name
            }),
            Record::Class(class) => {
                let class_type = &self.rec.class_types[class.class_type_id];
                json!({
                    "type": "Class",
                    "class_type_id": class.class_type_id,
                    "class_type_name": &class_type.name,
                    "members": self.encode_members(
                        &class.members,
                        &class_type.member_types,
                        &class_type.member_names
                    )
                })
            }
            Record::BinaryArray(member_type, vals) => json!({
                "type": "BinaryArray",
                "member_type": self.encode_member_type(member_type),
                "values": self.encode_member_values(vals, member_type)
            }),
            Record::PrimitiveArray(prim_type, vals) => json!({
                "type": "PrimitiveArray",
                "primitive_type": self.encode_primitive_type_name(prim_type),
                "values": self.encode_primitives(vals)
            }),
            Record::String(s) => json!({
                "type": "String",
                "value": s
            }),
        }
    }

    fn encode_members(
        &self,
        members: &[Member],
        types: &[MemberType],
        names: &[String],
    ) -> Vec<Value> {
        let mut result = Vec::new();
        let count = members.len().min(types.len()).min(names.len());
        for index in 0..count {
            let mut encoded = self.encode_member(&members[index], &types[index]);
            if let Value::Object(map) = &mut encoded {
                let display_name = Self::normalize_member_name(&names[index]);
                map.insert("name".to_string(), Value::String(display_name.to_string()));
            }
            result.push(encoded);
        }
        result
    }

    /// Normalizes a C# member name by stripping compiler-generated backing-field noise.
    fn normalize_member_name(name: &str) -> &str {
        if let Some(rest) = name.strip_prefix('<') {
            if let Some(inner_end) = rest.find('>') {
                let inner = &rest[..inner_end];
                // 显式接口实现：取最后一个分段
                return inner.rsplit('.').next().unwrap();
            }
        }
        name
    }

    fn encode_member(&self, member: &Member, _member_type: &MemberType) -> Value {
        match member {
            Member::Primitive(p) => self.encode_primitive(p),
            Member::Reference(id) => json!({
                "type": "Reference",
                "id": id
            }),
            Member::Null => json!({"type": "Null"}),
            Member::NullMultiple(count) => json!({
                "type": "NullMultiple",
                "count": count
            }),
        }
    }

    fn encode_primitive(&self, prim: &Primitive) -> Value {
        match prim {
            Primitive::Boolean(b) => json!({"type": "Boolean", "value": b}),
            Primitive::Byte(b) => json!({"type": "Byte", "value": b}),
            Primitive::Char(c) => json!({"type": "Char", "value": c.to_string()}),
            Primitive::Decimal(d) => json!({"type": "Decimal", "value": d}),
            Primitive::Double(d) => json!({"type": "Double", "value": d}),
            Primitive::Int16(i) => json!({"type": "Int16", "value": i}),
            Primitive::Int32(i) => json!({"type": "Int32", "value": i}),
            Primitive::Int64(i) => json!({"type": "Int64", "value": i}),
            Primitive::Int8(i) => json!({"type": "Int8", "value": i}),
            Primitive::Single(f) => json!({"type": "Single", "value": f}),
            Primitive::TimeSpan(i) => json!({"type": "TimeSpan", "value": i}),
            Primitive::DateTime(i) => json!({"type": "DateTime", "value": i}),
            Primitive::UInt16(u) => json!({"type": "UInt16", "value": u}),
            Primitive::UInt32(u) => json!({"type": "UInt32", "value": u}),
            Primitive::UInt64(u) => json!({"type": "UInt64", "value": u}),
            Primitive::Null => json!({"type": "Null"}),
            Primitive::String(s) => json!({"type": "String", "value": s}),
        }
    }

    fn encode_primitives(&self, prims: &[Primitive]) -> Vec<Value> {
        prims.iter().map(|p| self.encode_primitive(p)).collect()
    }

    fn encode_member_values(&self, members: &[Member], member_type: &MemberType) -> Vec<Value> {
        members
            .iter()
            .map(|m| self.encode_member(m, member_type))
            .collect()
    }

    fn encode_member_types(&self, types: &[MemberType]) -> Vec<Value> {
        types.iter().map(|t| self.encode_member_type(t)).collect()
    }

    fn encode_member_type(&self, typ: &MemberType) -> Value {
        match typ {
            MemberType::Primitive(p) => json!({
                "type": "Primitive",
                "primitive_type": self.encode_primitive_type_name(p)
            }),
            MemberType::String => json!({"type": "String"}),
            MemberType::Object => json!({"type": "Object"}),
            MemberType::SystemClass(name) => json!({"type": "SystemClass", "name": name}),
            MemberType::Class(name, lib_id) => json!({
                "type": "Class",
                "name": name,
                "library_id": lib_id
            }),
            MemberType::ObjectArray => json!({"type": "ObjectArray"}),
            MemberType::StringArray => json!({"type": "StringArray"}),
            MemberType::PrimitiveArray(p) => json!({
                "type": "PrimitiveArray",
                "primitive_type": self.encode_primitive_type_name(p)
            }),
        }
    }

    fn encode_primitive_type_name(&self, typ: &PrimitiveType) -> &str {
        match typ {
            PrimitiveType::Boolean => "Boolean",
            PrimitiveType::Byte => "Byte",
            PrimitiveType::Char => "Char",
            PrimitiveType::Decimal => "Decimal",
            PrimitiveType::Double => "Double",
            PrimitiveType::Int16 => "Int16",
            PrimitiveType::Int32 => "Int32",
            PrimitiveType::Int64 => "Int64",
            PrimitiveType::Int8 => "Int8",
            PrimitiveType::Single => "Single",
            PrimitiveType::TimeSpan => "TimeSpan",
            PrimitiveType::DateTime => "DateTime",
            PrimitiveType::UInt16 => "UInt16",
            PrimitiveType::UInt32 => "UInt32",
            PrimitiveType::UInt64 => "UInt64",
            PrimitiveType::Null => "Null",
            PrimitiveType::String => "String",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JsonEncoder;
    use crate::records::*;
    use std::collections::HashMap;

    fn make_simple_record(member_name: &str, value: i32) -> (DeserializedRecord, i32) {
        let class_type_id = 0;
        let record_id = 1;
        let class_types = vec![ClassType {
            name: "HeroStats".to_string(),
            library_id: 2,
            system_class: false,
            member_names: vec![member_name.to_string()],
            member_types: vec![MemberType::Primitive(PrimitiveType::Int32)],
        }];
        let mut records = HashMap::new();
        records.insert(
            record_id,
            Record::Class(Class {
                class_type_id,
                members: vec![Member::Primitive(Primitive::Int32(value))],
            }),
        );
        (
            DeserializedRecord {
                root_id: record_id,
                header_id: -1,
                records,
                class_types,
                record_metadata: HashMap::new(),
                record_order: vec![record_id],
            },
            record_id,
        )
    }

    #[test]
    fn normalize_member_name_leaves_plain_names_unchanged() {
        assert_eq!(JsonEncoder::normalize_member_name("maxSoldiers"), "maxSoldiers");
        assert_eq!(JsonEncoder::normalize_member_name("serializedVersion"), "serializedVersion");
        assert_eq!(JsonEncoder::normalize_member_name("id"), "id");
    }

    #[test]
    fn normalize_member_name_strips_auto_property_backing_field() {
        // C#自动属性
        assert_eq!(
            JsonEncoder::normalize_member_name("<maxSoldiers>k__BackingField"),
            "maxSoldiers"
        );
        assert_eq!(
            JsonEncoder::normalize_member_name("<squadLevel>k__BackingField"),
            "squadLevel"
        );
    }

    #[test]
    fn normalize_member_name_strips_explicit_interface_backing_field() {
        // C#显式接口实现
        assert_eq!(
            JsonEncoder::normalize_member_name(
                "<Voxels.TowerDefense.IHeroStats.soldiersLost>k__BackingField"
            ),
            "soldiersLost"
        );
        assert_eq!(
            JsonEncoder::normalize_member_name(
                "<Voxels.TowerDefense.IHeroStats.vikingsKilled>k__BackingField"
            ),
            "vikingsKilled"
        );
    }

    #[test]
    fn encode_members_uses_normalized_name_for_auto_property_backing_field() {
        let (rec, root_id) =
            make_simple_record("<maxSoldiers>k__BackingField", 8);
        let encoded = JsonEncoder::new(rec).encode();

        let records = &encoded["records"];
        let class_record = &records[root_id.to_string()];
        let members = class_record["members"].as_array().unwrap();
        assert_eq!(members[0]["name"], "maxSoldiers");
        assert_eq!(members[0]["value"], 8);
    }

    #[test]
    fn encode_members_uses_normalized_name_for_explicit_interface_backing_field() {
        let (rec, root_id) = make_simple_record(
            "<Voxels.TowerDefense.IHeroStats.soldiersLost>k__BackingField",
            3,
        );
        let encoded = JsonEncoder::new(rec).encode();

        let records = &encoded["records"];
        let class_record = &records[root_id.to_string()];
        let members = class_record["members"].as_array().unwrap();
        assert_eq!(members[0]["name"], "soldiersLost");
        assert_eq!(members[0]["value"], 3);
    }

    #[test]
    fn encode_class_types_preserves_original_backing_field_name_for_binary_roundtrip() {
        // 二进制往返所需保留原始C#名称
        let (rec, _) =
            make_simple_record("<maxSoldiers>k__BackingField", 8);
        let encoded = JsonEncoder::new(rec).encode();

        let class_types = encoded["class_types"].as_array().unwrap();
        let member_names = class_types[0]["member_names"].as_array().unwrap();
        assert_eq!(member_names[0], "<maxSoldiers>k__BackingField");
    }
}
