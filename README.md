# BadNorth 存档转换器 (BadNorthSaveConverter)

[English](#english) | [中文](#中文)

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.56%2B-blue.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/github/stars/ABaLaQiYaShanMaiI/BadNorthSaveConverter?style=social)](https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveConverter)

</div>

## 中文

### 项目介绍

**BadNorthSaveConverter** 是一个专业的 Rust 库和命令行工具，用于 *Bad North* 游戏存档文件的二进制与 JSON 格式之间的双向转换。该项目是 [BadNorthSaveModifier](https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveModifier) 的核心依赖库，提供了存档文件解析、序列化和转换的底层功能。

### 核心原理

#### 存档文件结构理解

Bad North 游戏使用 Unity 引擎，其存档文件采用 Unity 的序列化格式：

```
┌─────────────────────────────────────┐
│  Unity 序列化头信息                    │
│  - 元数据大小 (Metadata Size)          │
│  - 文件大小 (File Size)                │
│  - 版本号 (Version)                    │
│  - 数据偏移 (Data Offset)              │
└─────────────────────────────────────┘
        ↓
┌─────────────────────────────────────┐
│  序列化对象记录流                      │
│  一系列 Record 按特定格式排列          │
│  每个 Record 代表游戏数据结构          │
│  (英雄、升级、背包等)                 │
└─────────────────────────────────────┘
```

#### 转换流程

1. **二进制解析 (Binary Parsing)**
   - 按条记录读取二进制文件
   - 使用小端字节序 (Little Endian) 解析数据
   - 构建内存中的对象模型
   - 提取英雄、升级、背包等游戏数据

2. **数据模型化**
   ```
   SaveFile
   ├── UnityInfo (Unity 元数据)
   |   ├── metadataSize
   |   ├── fileSize
   |   ├── version
   |   └── dataOffset
   └── CampaignSave (游戏存档数据)
       ├── 进度信息 (levelStates, vikingFrontierPosition)
       ├── 英雄信息 (heroes, heroUpgrades)
       ├── 背包系统 (inventory)
       ├── 游戏统计 (stats, battleCount, turnCount)
       └── 玩家偏好 (prefs)
   ```

3. **JSON 序列化**
   - 将内存模型转换为可读的 JSON 格式
   - 支持大小写灵活转换（camelCase ↔ snake_case）
   - 保留所有游戏数据的完整性

4. **反向转换（JSON → 二进制）**
   - 从 JSON 加载数据
   - 重构 Unity 对象模型
   - 进行数据验证 (Checksum 验证)
   - 重新生成二进制文件

### 项目结构

```
BadNorthSaveConverter/
├── src/
│   ├── main.rs              # CLI 程序入口
│   ├── model.rs             # 游戏数据模型 (SaveFile, CampaignSave 等)
│   ├── parser.rs            # 二进制解析器
│   ├── records.rs           # 序列化记录定义
│   ├── json/
│   │   ├── mod.rs           # JSON 模块入口
│   │   ├── decoder.rs       # JSON 反序列化
│   │   └── encoder.rs       # JSON 序列化
│   └── serializer/
│       ├── mod.rs           # 序列化模块入口
│       ├── basic.rs         # 基础二进制序列化
│       ├── enhanced.rs      # 增强的序列化功能
│       ├── config.rs        # 序列化配置
│       └── string_patcher.rs # 字符串修补工具
├── Cargo.toml               # 项目依赖配置
└── README.md                # 项目文档
```

### 主要功能

#### 1. 二进制转 JSON (`bin2json`)
将游戏存档从二进制格式转换为可读的 JSON 格式，便于查看和编辑数据。

**优点：**
- 便于数据审查和调试
- JSON 格式便于手动修改
- 易于集成到其他工具

#### 2. JSON 转二进制 (`json2bin`)
将修改后的 JSON 文件转换回二进制格式，重新生成可用的游戏存档。

**关键特性：**
- 数据验证和完整性检查
- Checksum 校验确保数据正确性
- 支持多种序列化方法选择

#### 3. 库编程接口 (Library API)
作为库被其他项目使用，提供编程接口：
- `parse()` - 解析二进制存档
- `JsonDecoder` - JSON 解码
- `JsonEncoder` - JSON 编码
- `serialize_checked()` - 带验证的序列化

### 技术栈

| 技术 | 用途 |
|------|------|
| **Rust 1.56+** | 系统级语言，性能和安全性 |
| **serde** | 序列化/反序列化框架 |
| **serde_json** | JSON 支持 |
| **byteorder** | 字节序处理（Little Endian） |
| **clap** | 命令行参数解析 |

### 与 BadNorthSaveModifier 的关系

```graphql
操作流程示意图：

用户 GUI (BadNorthSaveModifier)
     ↓
     ├─→ 加载存档 → BadNorthSaveConverter.bin2json()
     │                      ↓
     │              在内存中操作 JSON 数据
     │
     ├─→ 保存存档 → BadNorthSaveConverter.json2bin()
     │                      ↓
     └─→ 生成新的二进制存档文件
```

**BadNorthSaveConverter 的作用：**
1. **数据入口** - 从游戏存档读取原始数据
2. **格式转换** - 提供二进制 ↔ JSON 转换
3. **数据验证** - 确保转换后的数据有效
4. **库支持** - BadNorthSaveModifier GUI 依赖本库进行数据处理

### 安装与编译

#### 前置要求
- Rust 1.56+ (推荐最新稳定版)
- Cargo

#### 作为命令行工具

1. **编译**
   ```bash
   cargo build --release
   ```

2. **可执行文件位置**
   ```
   target/release/BadNorthSaveConverter.exe
   ```

#### 作为库依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
BadNorthSaveConverter = { git = "https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveConverter.git" }
```

### 使用方法

#### 命令行使用

##### 1. 二进制转 JSON

```bash
# 指定输出文件
BadNorthSaveConverter bin2json save.dat save.json

# 自动生成输出文件名
BadNorthSaveConverter bin2json save.dat
# 生成: save.json
```

##### 2. JSON 转二进制

```bash
# 指定输出文件
BadNorthSaveConverter json2bin save.json save.dat

# 自动生成输出文件名
BadNorthSaveConverter json2bin save.json
# 生成: save.dat
```

#### 库编程使用

```rust
use BadNorthSaveConverter::parser::parse;
use BadNorthSaveConverter::json::JsonEncoder;
use std::fs;

// 1. 读取二进制存档
let binary_data = fs::read("save.dat")?;

// 2. 解析为内存模型
let deserialized = parse(&binary_data)?;

// 3. 转换为 JSON
let json_encoder = JsonEncoder::new(deserialized);
let json_string = json_encoder.encode()?;

// 4. 保存 JSON
fs::write("save.json", json_string)?;
```

### 数据格式示例

#### JSON 中的存档结构

```json
{
  "unity": {
    "metadata_size": 1234,
    "file_size": 56789,
    "version": 9,
    "data_offset": 4096
  },
  "campaign": {
    "serialized_version": 2,
    "seed": 12345,
    "level_states": [...],
    "heroes": [
      {
        "heroId": 1,
        "level": 5,
        "experience": 1000,
        "skills": [...]
      }
    ],
    "inventory": [...],
    "coin_bank": 500,
    "turn_count": 42,
    "battle_count": 15,
    "battles_won": 12,
    "stats": {...},
    "prefs": {...}
  }
}
```

### 关键模块说明

#### `parser.rs` - 二进制解析器
- 按 Unity 序列化格式读取二进制文件
- 构建对象关系图
- 提取结构化数据

#### `serializer/basic.rs` - 基础序列化
- 生成 Unity 兼容的二进制格式
- 正确的字节序和对齐
- Checksum 计算和验证

#### `serializer/enhanced.rs` - 增强功能
- 高级数据转换
- 优化和压缩选项

#### `json/decoder.rs` - JSON 反序列化
- 将 JSON 转换为内存对象
- 类型转换和验证

#### `json/encoder.rs` - JSON 序列化
- 将对象转换为 JSON
- 格式化和缩进选项

### 数据校验

为了确保数据转换的正确性，本项目实现了以下校验机制：

1. **结构完整性检查** - 验证所有必需字段
2. **数据类型检查** - 确保类型转换正确
3. **Checksum 验证** - 验证二进制数据完整性
4. **范围检查** - 确保数值在有效范围内

### 常见用例

#### 用例 1: 辅助修改工具
```
原始存档 (save.dat)
  ↓
BadNorthSaveConverter (bin2json)
  ↓
JSON 文件 (save.json) ← 可用任何工具编辑
  ↓
BadNorthSaveConverter (json2bin)
  ↓
修改后的存档 (save_modified.dat)
```

#### 用例 2: 集成到 GUI 应用
BadNorthSaveModifier 集成本库，提供友好的 GUI 界面：
```
GUI 界面读取存档
  → 调用 bin2json() 获取 JSON
  → 显示可编辑的数据
  → 用户修改数据
  → 调用 json2bin() 保存存档
```

#### 用例 3: 存档分析和统计
```
多个存档文件
  → 批量转换为 JSON
  → 分析游戏统计数据
  → 生成报告
```

### 限制和注意事项

- ⚠️ 仅支持 Bad North 游戏存档格式
- ⚠️ 修改前建议备份原始存档
- ⚠️ 某些数据字段值需要符合游戏逻辑约束
- ⚠️ 大型存档文件处理可能消耗较多内存

### 故障排除

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| 无法识别的存档格式 | 文件已损坏或非 Bad North 存档 | 检查文件完整性；尝试恢复备份 |
| JSON 解析错误 | JSON 格式不正确或包含非法值 | 验证 JSON 格式；检查数据类型 |
| 二进制生成失败 | Checksum 不匹配 | 使用原始完整的 JSON 数据 |
| 字符编码问题 | 特殊字符处理不当 | 确保使用 UTF-8 编码 |

### 开发贡献

欢迎提交 Issue 和 Pull Request！

### 许可证

本项目基于 **MIT License** 开源，详见 [LICENSE](./LICENSE) 文件。

### 作者

**ABaLaQiYaShanMaiI**

### 相关项目

- [BadNorthSaveModifier](https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveModifier) - GUI 编辑工具（依赖本项目）
- [Bad North](https://www.badnorth.com/) - 官方游戏网站

---

## English

### Project Description

**BadNorthSaveConverter** is a professional Rust library and command-line tool for bidirectional conversion between binary and JSON formats for *Bad North* game save files. This project is the core dependency library of [BadNorthSaveModifier](https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveModifier), providing low-level functionality for save file parsing, serialization, and conversion.

### Core Principles

#### Understanding Save File Structure

Bad North uses the Unity engine, and its save files use Unity's serialization format:

```
┌─────────────────────────────────────┐
│  Unity Serialization Header          │
│  - Metadata Size                      │
│  - File Size                          │
│  - Version                            │
│  - Data Offset                        │
└─────────────────────────────────────┘
        ↓
┌─────────────────────────────────────┐
│  Serialized Object Record Stream     │
│  Series of Records in specific format│
│  Each Record represents game data    │
│  (Heroes, Upgrades, Inventory, etc.) │
└─────────────────────────────────────┘
```

#### Conversion Pipeline

1. **Binary Parsing**
   - Read records sequentially from binary file
   - Parse data using little-endian byte order
   - Build in-memory object model
   - Extract game data (heroes, upgrades, inventory)

2. **Data Model**
   ```
   SaveFile
   ├── UnityInfo (Unity metadata)
   |   ├── metadataSize
   |   ├── fileSize
   |   ├── version
   |   └── dataOffset
   └── CampaignSave (Game save data)
       ├── Progress info (levelStates, vikingFrontierPosition)
       ├── Hero info (heroes, heroUpgrades)
       ├── Inventory system (inventory)
       ├── Game stats (stats, battleCount, turnCount)
       └── Player preferences (prefs)
   ```

3. **JSON Serialization**
   - Convert in-memory model to readable JSON format
   - Support flexible case conversion (camelCase ↔ snake_case)
   - Preserve complete game data integrity

4. **Reverse Conversion (JSON → Binary)**
   - Load data from JSON
   - Reconstruct Unity object model
   - Perform data validation (Checksum verification)
   - Regenerate binary file

### Project Structure

```
BadNorthSaveConverter/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── model.rs             # Game data models (SaveFile, CampaignSave, etc.)
│   ├── parser.rs            # Binary parser
│   ├── records.rs           # Serialization record definitions
│   ├── json/
│   │   ├── mod.rs           # JSON module entry
│   │   ├── decoder.rs       # JSON deserialization
│   │   └── encoder.rs       # JSON serialization
│   └── serializer/
│       ├── mod.rs           # Serializer module entry
│       ├── basic.rs         # Basic binary serialization
│       ├── enhanced.rs      # Enhanced serialization features
│       ├── config.rs        # Serialization configuration
│       └── string_patcher.rs # String patching utilities
├── Cargo.toml               # Project dependencies
└── README.md                # Project documentation
```

### Key Features

#### 1. Binary to JSON (`bin2json`)
Convert game saves from binary format to readable JSON format for easy viewing and editing.

**Benefits:**
- Easy data inspection and debugging
- JSON format suitable for manual editing
- Easy integration with other tools

#### 2. JSON to Binary (`json2bin`)
Convert modified JSON files back to binary format to regenerate usable game saves.

**Key Features:**
- Data validation and integrity checks
- Checksum verification for data correctness
- Support for multiple serialization methods

#### 3. Library Programming Interface (API)
Used as a library by other projects, providing programming interfaces:
- `parse()` - Parse binary save
- `JsonDecoder` - JSON decoding
- `JsonEncoder` - JSON encoding
- `serialize_checked()` - Serialization with validation

### Tech Stack

| Technology | Purpose |
|-----------|---------|
| **Rust 1.56+** | System language, performance and safety |
| **serde** | Serialization/deserialization framework |
| **serde_json** | JSON support |
| **byteorder** | Byte order handling (Little Endian) |
| **clap** | Command-line argument parsing |

### Relationship with BadNorthSaveModifier

```graphql
Operation Flow:

User GUI (BadNorthSaveModifier)
     ↓
     ├─→ Load save → BadNorthSaveConverter.bin2json()
     │                      ↓
     │              Manipulate JSON data in memory
     │
     ├─→ Save → BadNorthSaveConverter.json2bin()
     │                      ↓
     └─→ Generate new binary save file
```

**BadNorthSaveConverter's Role:**
1. **Data Entry Point** - Read raw data from game saves
2. **Format Conversion** - Provide binary ↔ JSON conversion
3. **Data Validation** - Ensure converted data is valid
4. **Library Support** - BadNorthSaveModifier GUI depends on this library for data processing

### Installation & Compilation

#### Prerequisites
- Rust 1.56+ (latest stable recommended)
- Cargo

#### As Command-Line Tool

1. **Build**
   ```bash
   cargo build --release
   ```

2. **Executable Location**
   ```
   target/release/BadNorthSaveConverter.exe
   ```

#### As Library Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
BadNorthSaveConverter = { git = "https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveConverter.git" }
```

### Usage

#### Command-Line Usage

##### 1. Convert Binary to JSON

```bash
# Specify output file
BadNorthSaveConverter bin2json save.dat save.json

# Auto-generate output filename
BadNorthSaveConverter bin2json save.dat
# Generates: save.json
```

##### 2. Convert JSON to Binary

```bash
# Specify output file
BadNorthSaveConverter json2bin save.json save.dat

# Auto-generate output filename
BadNorthSaveConverter json2bin save.json
# Generates: save.dat
```

#### Library Usage

```rust
use BadNorthSaveConverter::parser::parse;
use BadNorthSaveConverter::json::JsonEncoder;
use std::fs;

// 1. Read binary save
let binary_data = fs::read("save.dat")?;

// 2. Parse into memory model
let deserialized = parse(&binary_data)?;

// 3. Convert to JSON
let json_encoder = JsonEncoder::new(deserialized);
let json_string = json_encoder.encode()?;

// 4. Save JSON
fs::write("save.json", json_string)?;
```

### Data Format Example

#### Save Structure in JSON

```json
{
  "unity": {
    "metadata_size": 1234,
    "file_size": 56789,
    "version": 9,
    "data_offset": 4096
  },
  "campaign": {
    "serialized_version": 2,
    "seed": 12345,
    "level_states": [...],
    "heroes": [
      {
        "heroId": 1,
        "level": 5,
        "experience": 1000,
        "skills": [...]
      }
    ],
    "inventory": [...],
    "coin_bank": 500,
    "turn_count": 42,
    "battle_count": 15,
    "battles_won": 12,
    "stats": {...},
    "prefs": {...}
  }
}
```

### Key Module Details

#### `parser.rs` - Binary Parser
- Read binary files according to Unity serialization format
- Build object relation graph
- Extract structured data

#### `serializer/basic.rs` - Basic Serialization
- Generate Unity-compatible binary format
- Correct byte ordering and alignment
- Checksum calculation and verification

#### `serializer/enhanced.rs` - Enhanced Features
- Advanced data transformation
- Optimization and compression options

#### `json/decoder.rs` - JSON Deserialization
- Convert JSON to in-memory objects
- Type conversion and validation

#### `json/encoder.rs` - JSON Serialization
- Convert objects to JSON
- Formatting and indentation options

### Data Validation

The project implements the following validation mechanisms to ensure conversion correctness:

1. **Structure Integrity Checking** - Verify all required fields
2. **Data Type Checking** - Ensure type conversions are correct
3. **Checksum Verification** - Verify binary data integrity
4. **Range Checking** - Ensure values are within valid ranges

### Common Use Cases

#### Use Case 1: Modification Helper
```
Original save (save.dat)
  ↓
BadNorthSaveConverter (bin2json)
  ↓
JSON file (save.json) ← Can be edited with any tool
  ↓
BadNorthSaveConverter (json2bin)
  ↓
Modified save (save_modified.dat)
```

#### Use Case 2: Integration into GUI Application
BadNorthSaveModifier integrates this library to provide a friendly GUI interface:
```
GUI reads save
  → Call bin2json() to get JSON
  → Display editable data
  → User modifies data
  → Call json2bin() to save
```

#### Use Case 3: Save Analysis and Statistics
```
Multiple save files
  → Batch convert to JSON
  → Analyze game statistics
  → Generate reports
```

### Limitations and Considerations

- ⚠️ Only supports Bad North game save format
- ⚠️ Backup original saves before modification
- ⚠️ Certain data field values must meet game logic constraints
- ⚠️ Large save files may consume significant memory

### Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Unrecognized save format | File is corrupted or not a Bad North save | Check file integrity; try recovering from backup |
| JSON parsing error | JSON format is incorrect or contains invalid values | Validate JSON format; check data types |
| Binary generation fails | Checksum mismatch | Use original complete JSON data |
| Character encoding issues | Special characters not handled properly | Ensure UTF-8 encoding is used |

### Contributing

Issues and Pull Requests are welcome!

### License

This project is open-sourced under the **MIT License**. See the [LICENSE](./LICENSE) file for details.

### Author

**ABaLaQiYaShanMaiI**

### Related Projects

- [BadNorthSaveModifier](https://github.com/ABaLaQiYaShanMaiI/BadNorthSaveModifier) - GUI editing tool (depends on this project)
- [Bad North](https://www.badnorth.com/) - Official game website
