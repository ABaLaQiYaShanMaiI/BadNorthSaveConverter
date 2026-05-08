/// Serializer configuration module
/// 序列化器配置模块
///
/// 提供灵活的序列化模式和配置选项

use std::fmt;

/// 序列化模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SerializerMode {
    /// 严格模式：完全遵循二进制格式，任何变化都拒绝
    /// 所有记录的大小必须与原始二进制完全相同
    Strict,

    /// 灵活字符串模式（推荐用于 modifier）
    /// 允许字符串记录（type 6）的大小变化
    /// 其他记录类型仍然严格验证
    /// 字符串记录是自描述的（带有 7-bit 编码长度前缀），所以可以安全地改变大小
    FlexibleStrings,

    /// 自适应模式（未来扩展）
    /// 根据内容自动选择最佳模式
    Adaptive,
}

impl fmt::Display for SerializerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializerMode::Strict => write!(f, "Strict"),
            SerializerMode::FlexibleStrings => write!(f, "FlexibleStrings"),
            SerializerMode::Adaptive => write!(f, "Adaptive"),
        }
    }
}

/// 序列化器配置
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SerializerConfig {
    /// 序列化模式
    pub mode: SerializerMode,

    /// 是否输出详细的诊断信息
    pub verbose_diagnostics: bool,

    /// 是否允许缺失元数据
    /// 某些记录可能没有对应的元数据，该选项控制是否允许
    pub allow_missing_metadata: bool,

    /// 是否记录和输出序列化摘要
    pub log_record_summary: bool,
}

impl Default for SerializerConfig {
    fn default() -> Self {
        SerializerConfig {
            mode: SerializerMode::FlexibleStrings,
            verbose_diagnostics: false,
            allow_missing_metadata: true,
            log_record_summary: false,
        }
    }
}

impl SerializerConfig {
    /// 为 modifier 场景创建默认配置
    /// 推荐用于接受 modifier 修改的 JSON 的序列化
    #[allow(dead_code)]
    pub fn default_for_modifier() -> Self {
        SerializerConfig {
            mode: SerializerMode::FlexibleStrings,
            verbose_diagnostics: true,
            allow_missing_metadata: true,
            log_record_summary: true,
        }
    }

    /// 创建严格模式配置
    /// 用于精确复制和测试，要求完全相同的二进制
    #[allow(dead_code)]
    pub fn strict() -> Self {
        SerializerConfig {
            mode: SerializerMode::Strict,
            verbose_diagnostics: false,
            allow_missing_metadata: false,
            log_record_summary: false,
        }
    }

    /// 创建详细诊断模式配置
    /// 输出所有变化和诊断信息
    #[allow(dead_code)]
    pub fn verbose() -> Self {
        SerializerConfig {
            mode: SerializerMode::FlexibleStrings,
            verbose_diagnostics: true,
            allow_missing_metadata: true,
            log_record_summary: true,
        }
    }

    /// 创建适应模式配置
    #[allow(dead_code)]
    pub fn adaptive() -> Self {
        SerializerConfig {
            mode: SerializerMode::Adaptive,
            verbose_diagnostics: false,
            allow_missing_metadata: true,
            log_record_summary: false,
        }
    }

    /// 设置序列化模式
    #[allow(dead_code)]
    pub fn with_mode(mut self, mode: SerializerMode) -> Self {
        self.mode = mode;
        self
    }

    /// 启用/禁用详细诊断
    #[allow(dead_code)]
    pub fn with_verbose_diagnostics(mut self, verbose: bool) -> Self {
        self.verbose_diagnostics = verbose;
        self
    }

    /// 启用/禁用缺失元数据允许
    #[allow(dead_code)]
    pub fn with_allow_missing_metadata(mut self, allow: bool) -> Self {
        self.allow_missing_metadata = allow;
        self
    }

    /// 启用/禁用记录摘要
    #[allow(dead_code)]
    pub fn with_log_record_summary(mut self, log: bool) -> Self {
        self.log_record_summary = log;
        self
    }
}

impl fmt::Display for SerializerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SerializerConfig {{ mode: {}, verbose: {}, allow_missing: {}, summary: {} }}",
            self.mode, self.verbose_diagnostics, self.allow_missing_metadata, self.log_record_summary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SerializerConfig::default();
        assert_eq!(config.mode, SerializerMode::FlexibleStrings);
        assert!(!config.verbose_diagnostics);
        assert!(config.allow_missing_metadata);
        assert!(!config.log_record_summary);
    }

    #[test]
    fn test_modifier_config() {
        let config = SerializerConfig::default_for_modifier();
        assert_eq!(config.mode, SerializerMode::FlexibleStrings);
        assert!(config.verbose_diagnostics);
        assert!(config.allow_missing_metadata);
        assert!(config.log_record_summary);
    }

    #[test]
    fn test_strict_config() {
        let config = SerializerConfig::strict();
        assert_eq!(config.mode, SerializerMode::Strict);
        assert!(!config.verbose_diagnostics);
        assert!(!config.allow_missing_metadata);
        assert!(!config.log_record_summary);
    }

    #[test]
    fn test_config_builder() {
        let config = SerializerConfig::default()
            .with_mode(SerializerMode::Strict)
            .with_verbose_diagnostics(true);

        assert_eq!(config.mode, SerializerMode::Strict);
        assert!(config.verbose_diagnostics);
    }
}
