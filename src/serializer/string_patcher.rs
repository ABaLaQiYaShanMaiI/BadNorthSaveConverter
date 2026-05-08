use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum StringChangeType {
    /// 字符串变短了
    Shortened,
    /// 字符串变长了
    Lengthened,
    /// 内容改变但长度可能相同
    Modified,
    /// 没有变化
    Unchanged,
}

/// 字符串补丁信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StringPatchInfo {
    /// 记录 ID
    pub record_id: i32,
    /// 原始值
    pub old_value: String,
    /// 新值
    pub new_value: String,
    /// 变化类型
    pub change_type: StringChangeType,
    /// 字节差异（新-旧）
    pub size_delta: isize,
}

impl StringPatchInfo {
    /// 创建新的字符串补丁信息
    #[allow(dead_code)]
    pub fn new(record_id: i32, old_value: String, new_value: String) -> Self {
        let old_size = calculate_string_record_size(&old_value);
        let new_size = calculate_string_record_size(&new_value);
        let size_delta = new_size as isize - old_size as isize;

        let change_type = if old_value == new_value {
            StringChangeType::Unchanged
        } else if new_size < old_size {
            StringChangeType::Shortened
        } else if new_size > old_size {
            StringChangeType::Lengthened
        } else {
            StringChangeType::Modified
        };

        StringPatchInfo {
            record_id,
            old_value,
            new_value,
            change_type,
            size_delta,
        }
    }

    /// 生成诊断消息
    #[allow(dead_code)]
    pub fn diagnostic_message(&self) -> String {
        let old_size = calculate_string_record_size(&self.old_value);
        let new_size = calculate_string_record_size(&self.new_value);

        match self.change_type {
            StringChangeType::Unchanged => {
                format!(
                    "  • Record {}: No change ('{}', {} bytes)",
                    self.record_id, self.old_value, old_size
                )
            }
            StringChangeType::Shortened => {
                format!(
                    "  • Record {}: Shortened '{}' ({} bytes) → '{}' ({} bytes), Δ={} bytes",
                    self.record_id, self.old_value, old_size, self.new_value, new_size, self.size_delta
                )
            }
            StringChangeType::Lengthened => {
                format!(
                    "  • Record {}: Lengthened '{}' ({} bytes) → '{}' ({} bytes), Δ=+{} bytes",
                    self.record_id, self.old_value, old_size, self.new_value, new_size, self.size_delta.abs()
                )
            }
            StringChangeType::Modified => {
                if self.size_delta == 0 {
                    format!(
                        "  • Record {}: Modified '{}' → '{}' (same size, {} bytes)",
                        self.record_id, self.old_value, self.new_value, new_size
                    )
                } else {
                    format!(
                        "  • Record {}: Modified '{}' ({} bytes) → '{}' ({} bytes), Δ={} bytes",
                        self.record_id,
                        self.old_value,
                        old_size,
                        self.new_value,
                        new_size,
                        self.size_delta
                    )
                }
            }
        }
    }
}

impl fmt::Display for StringPatchInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagnostic_message())
    }
}

/// 计算字符串在记录中的编码大小
/// 包括 7-bit 编码的长度前缀
#[allow(dead_code)]
pub fn calculate_string_record_size(string_value: &str) -> usize {
    let length = string_value.len();
    let prefix_len = match length {
        0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1F_FFFF => 3,
        0x20_0000..=0x0FFF_FFFF => 4,
        _ => 5,
    };
    prefix_len + length
}

/// 计算字符串变化的字节差异
#[allow(dead_code)]
pub fn calculate_size_delta(old_str: &str, new_str: &str) -> isize {
    let old_size = calculate_string_record_size(old_str);
    let new_size = calculate_string_record_size(new_str);
    new_size as isize - old_size as isize
}

/// 检测字符串变化类型
#[allow(dead_code)]
pub fn detect_change_type(old_str: &str, new_str: &str) -> StringChangeType {
    if old_str == new_str {
        StringChangeType::Unchanged
    } else {
        let old_size = calculate_string_record_size(old_str);
        let new_size = calculate_string_record_size(new_str);

        if new_size < old_size {
            StringChangeType::Shortened
        } else if new_size > old_size {
            StringChangeType::Lengthened
        } else {
            StringChangeType::Modified
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_string_record_size() {
        // 0-127 字节：1 字节前缀
        let s = "test"; // 4 字节
        assert_eq!(calculate_string_record_size(s), 5); // 1 + 4

        // 128-16383 字节：2 字节前缀
        let s = "a".repeat(128); // 128 字节
        assert_eq!(calculate_string_record_size(&s), 130); // 2 + 128
    }

    #[test]
    fn test_string_patch_info() {
        let patch = StringPatchInfo::new(
            100,
            "Hero_Class_Infantry".to_string(),
            "Hero_Class_Archer".to_string(),
        );

        assert_eq!(patch.record_id, 100);
        assert_eq!(patch.change_type, StringChangeType::Shortened);
        assert!(patch.size_delta < 0);
    }

    #[test]
    fn test_detect_change_type() {
        assert_eq!(
            detect_change_type("hello", "hello"),
            StringChangeType::Unchanged
        );

        assert_eq!(
            detect_change_type("hello", "hi"),
            StringChangeType::Shortened
        );

        assert_eq!(
            detect_change_type("hi", "hello"),
            StringChangeType::Lengthened
        );
    }

    #[test]
    fn test_size_delta_calculation() {
        let delta = calculate_size_delta("Infantry", "Archer");
        assert!(delta < 0); // Archer 更短
    }
}
