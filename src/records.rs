use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct DeserializedRecord {
    pub root_id: i32,
    pub header_id: i32,
    pub records: HashMap<i32, Record>,
    pub class_types: Vec<ClassType>,
    pub record_metadata: HashMap<i32, RecordMetadata>,
    pub record_order: Vec<i32>,
}

impl DeserializedRecord {
    pub fn class_type(&self, class: &Class) -> &ClassType {
        &self.class_types[class.class_type_id]
    }

    #[allow(dead_code)]
    pub fn class_member<'a, 'b>(&'a self, class: &'a Class, name: &'b str) -> &'a Member {
        &class.members[self.class_member_index(class, name)]
    }

    #[allow(dead_code)]
    pub fn class_member_mut<'a, 'b>(
        &'a mut self,
        class: &'a mut Class,
        name: &'b str,
    ) -> &'a mut Member {
        let index = self.class_member_index(class, name);
        &mut class.members[index]
    }

    #[allow(dead_code)]
    pub fn patch_class_member(
        &mut self,
        class_record_id: i32,
        name: &str,
        member: Member,
    ) -> Result<(), String> {
        let class_type_id = match self.records.get(&class_record_id) {
            Some(Record::Class(class)) => class.class_type_id,
            Some(_) => return Err(format!("Record {} is not a Class", class_record_id)),
            None => return Err(format!("Missing class record {}", class_record_id)),
        };
        let member_index = self.class_types[class_type_id]
            .member_names
            .iter()
            .position(|member_name| member_name == name)
            .ok_or_else(|| {
                format!(
                    "Member '{}' not found in class '{}'",
                    name, self.class_types[class_type_id].name
                )
            })?;
        let class = self.records.get_mut(&class_record_id).unwrap().as_class_mut();
        if class.members.len() != self.class_types[class_type_id].member_types.len() {
            return Err(format!(
                "Class record {} member count mismatch: expected {}, got {}",
                class_record_id,
                self.class_types[class_type_id].member_types.len(),
                class.members.len()
            ));
        }
        class.members[member_index] = member;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn class_member_deref<'a, 'b>(&'a self, class: &'a Class, name: &'b str) -> &'a Record {
        let id = self.class_member(class, name).as_reference();
        &self.records[id]
    }

    #[allow(dead_code)]
    pub fn class_member_index<'a, 'b>(&'a self, class: &'a Class, name: &'b str) -> usize {
        let class_type = self.class_type(class);
        class_type
            .member_names
            .iter()
            .position(|n| n == name)
            .unwrap()
    }

    pub fn record_metadata(&self, record_id: i32) -> Option<&RecordMetadata> {
        self.record_metadata.get(&record_id)
    }

    pub fn validate_graph(&self) -> Result<(), String> {
        if !self.records.contains_key(&self.root_id) {
            return Err(format!("Missing root record {}", self.root_id));
        }
        for (record_id, record) in &self.records {
            match record {
                Record::Class(class) => {
                    let Some(class_type) = self.class_types.get(class.class_type_id) else {
                        return Err(format!(
                            "Record {} uses invalid class_type_id {}",
                            record_id, class.class_type_id
                        ));
                    };
                    if class.members.len() != class_type.member_types.len() {
                        return Err(format!(
                            "Record {} class '{}' member count mismatch: expected {}, got {}",
                            record_id,
                            class_type.name,
                            class_type.member_types.len(),
                            class.members.len()
                        ));
                    }
                    for member in &class.members {
                        self.validate_member_reference(*record_id, member)?;
                    }
                }
                Record::BinaryArray(_, values) => {
                    for member in values {
                        self.validate_member_reference(*record_id, member)?;
                    }
                }
                Record::BinaryLibrary(_) | Record::PrimitiveArray(_, _) | Record::String(_) => {}
            }
        }

        Ok(())
    }

    fn validate_member_reference(&self, owner_record_id: i32, member: &Member) -> Result<(), String> {
        if let Member::Reference(reference_id) = member {
            if !self.records.contains_key(reference_id) {
                return Err(format!(
                    "Record {} references missing record {}",
                    owner_record_id, reference_id
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordMetadata {
    pub original_record_type: u8,
    pub start_offset: usize,
    pub end_offset: usize,
    pub original_byte_length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    BinaryLibrary(String),
    Class(Class),
    BinaryArray(MemberType, Vec<Member>),
    PrimitiveArray(PrimitiveType, Vec<Primitive>),
    String(String),
}

impl Record {
    #[allow(dead_code)]
    pub const fn as_class(&self) -> &Class {
        if let Self::Class(class) = self {
            class
        } else {
            panic!("记录不是Class")
        }
    }

    #[allow(dead_code)]
    pub fn as_binary_array(&self) -> &[Member] {
        if let Self::BinaryArray(_, array) = self {
            array
        } else {
            panic!("记录不是BinaryArray")
        }
    }

    #[allow(dead_code)]
    pub fn as_class_mut(&mut self) -> &mut Class {
        if let Self::Class(class) = self {
            class
        } else {
            panic!("记录不是Class")
        }
    }

    #[allow(dead_code)]
    pub fn as_binary_array_mut(&mut self) -> &mut Vec<Member> {
        if let Self::BinaryArray(_, array) = self {
            array
        } else {
            panic!("记录不是BinaryArray")
        }
    }

    #[allow(dead_code)]
    pub fn as_string(&self) -> &str {
        if let Self::String(s) = self {
            s
        } else {
            panic!("记录不是String")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub class_type_id: usize,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassType {
    pub name: String,
    pub library_id: i32,
    pub system_class: bool,
    pub member_names: Vec<String>,
    pub member_types: Vec<MemberType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemberType {
    Primitive(PrimitiveType),
    String,
    Object,
    SystemClass(String),
    Class(String, i32),
    ObjectArray,
    StringArray,
    PrimitiveArray(PrimitiveType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Primitive(Primitive),
    Reference(i32),
    Null,
    NullMultiple(i32),
}

impl Member {
    #[allow(dead_code)]
    pub const fn as_reference(&self) -> &i32 {
        if let Self::Reference(id) = self {
            id
        } else {
            panic!("成员不是引用")
        }
    }

    #[allow(dead_code)]
    pub const fn as_i32(&self) -> i32 {
        if let Self::Primitive(Primitive::Int32(val)) = self {
            *val
        } else {
            panic!("成员不是Int32")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Boolean,
    Byte,
    Char,
    Decimal,
    Double,
    Int16,
    Int32,
    Int64,
    Int8,
    Single,
    TimeSpan,
    DateTime,
    UInt16,
    UInt32,
    UInt64,
    Null,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Boolean(bool),
    Byte(u8),
    Char(char),
    Decimal(String),
    Double(f64),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int8(i8),
    Single(f32),
    TimeSpan(i64),
    DateTime(i64),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Null,
    String(String),
}

impl Primitive {
    pub const fn primitive_type(&self) -> PrimitiveType {
        match self {
            Primitive::Boolean(..) => PrimitiveType::Boolean,
            Primitive::Byte(..) => PrimitiveType::Byte,
            Primitive::Char(..) => PrimitiveType::Char,
            Primitive::Decimal(..) => PrimitiveType::Decimal,
            Primitive::Double(..) => PrimitiveType::Double,
            Primitive::Int16(..) => PrimitiveType::Int16,
            Primitive::Int32(..) => PrimitiveType::Int32,
            Primitive::Int64(..) => PrimitiveType::Int64,
            Primitive::Int8(..) => PrimitiveType::Int8,
            Primitive::Single(..) => PrimitiveType::Single,
            Primitive::TimeSpan(..) => PrimitiveType::TimeSpan,
            Primitive::DateTime(..) => PrimitiveType::DateTime,
            Primitive::UInt16(..) => PrimitiveType::UInt16,
            Primitive::UInt32(..) => PrimitiveType::UInt32,
            Primitive::UInt64(..) => PrimitiveType::UInt64,
            Primitive::Null => PrimitiveType::Null,
            Primitive::String(..) => PrimitiveType::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn patch_class_member_by_name_updates_member_in_place() {
        let mut graph = DeserializedRecord {
            root_id: 1,
            header_id: -1,
            records: HashMap::from([(
                1,
                Record::Class(Class {
                    class_type_id: 0,
                    members: vec![Member::Primitive(Primitive::Int32(10))],
                }),
            )]),
            class_types: vec![ClassType {
                name: "CampaignSave".to_string(),
                library_id: 2,
                system_class: false,
                member_names: vec!["coinBank".to_string()],
                member_types: vec![MemberType::Primitive(PrimitiveType::Int32)],
            }],
            record_metadata: HashMap::new(),
            record_order: vec![1],
        };

        graph
            .patch_class_member(1, "coinBank", Member::Primitive(Primitive::Int32(99)))
            .expect("patch should succeed");

        let class = graph.records[&1].as_class();
        assert_eq!(class.members[0], Member::Primitive(Primitive::Int32(99)));
    }

    #[test]
    fn validate_graph_rejects_missing_reference_record() {
        let graph = DeserializedRecord {
            root_id: 1,
            header_id: -1,
            records: HashMap::from([(
                1,
                Record::Class(Class {
                    class_type_id: 0,
                    members: vec![Member::Reference(42)],
                }),
            )]),
            class_types: vec![ClassType {
                name: "CampaignSave".to_string(),
                library_id: 2,
                system_class: false,
                member_names: vec!["stats".to_string()],
                member_types: vec![MemberType::Object],
            }],
            record_metadata: HashMap::new(),
            record_order: vec![1],
        };

        let error = graph.validate_graph().expect_err("graph should be invalid");
        assert!(error.contains("references missing record 42"));
    }
}
