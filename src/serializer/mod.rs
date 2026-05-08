// 序列化模块
// 提供二进制序列化功能

pub mod basic;
pub mod enhanced;
pub mod config;
pub mod string_patcher;

pub use basic::serialize_checked;
