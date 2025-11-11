# Claude Code Switcher

一个用于管理 Claude Code 设置快照和模板的 CLI 工具，让你轻松在不同的 AI 提供商之间切换。

## 🎯 核心特性

- 🔄 **一键切换**：在多个 AI 提供商之间无缝切换
- 📦 **预设模板**：内置 DeepSeek、智谱 GLM、MiniMax、Kimi 等热门 AI 提供商模板
- 💾 **快照功能**：保存和恢复你的配置组合（支持 env/common/all 三种作用域）
- 🌍 **环境隔离**：项目级和全局配置分离存储
- 🔐 **安全存储**：加密保存 API 密钥，支持多凭证管理
- 🧩 **模块化架构**：易于扩展支持新的 AI 提供商

## 🚀 快速开始

### 安装

#### 方法一：从 crates.io 安装
```bash
cargo install claude-code-switcher
```

#### 方法二：从源码安装
```bash
git clone https://github.com/Cupnfish/claude-code-switcher.git
cd claude-code-switcher
cargo install --path .
```

安装完成后，使用 `ccs --help` 验证安装：

```bash
ccs --help
```

## 💡 基本使用

### 1. 应用预设模板（推荐）

```bash
# 智谱 GLM（强烈推荐） - 支持 256K 上下文
ccs apply zai
# 别名：glm, zhipu

# MiniMax - Anthropic 兼容，性能出色
ccs apply minimax

# DeepSeek - 高性价比，响应快速
ccs apply deepseek
# 别名：ds

# Kimi For Coding - 专注编程场景
ccs apply kimi

# Moonshot K2 - 大上下文平衡性能
ccs apply k2
# 别名：moonshot

# K2 Thinking - 高速推理
ccs apply k2-thinking

# KatCoder Pro - 专业编程 AI
ccs apply kat-coder-pro
# 别名：katpro

# KatCoder Air - 轻量级快速响应
ccs apply kat-coder-air
# 别名：katair
```

**凭证管理**：首次使用模板时，工具会提示输入 API 密钥。选择保存后，密钥会存储在本地，下次使用自动加载。

### 2. 管理凭证

```bash
# 列出所有保存的凭证
ccs credentials list
# 或简写：ccs creds list

# 删除指定凭证（使用 ID）
ccs credentials delete <credential-id>

# 清除所有凭证
ccs credentials clear
```

### 3. 快照管理

```bash
# 创建当前设置的快照
ccs snap my-debug-config

# 查看所有快照
ccs ls -v

# 应用快照
ccs apply my-debug-config

# 删除快照
ccs delete my-debug-config
```

## 🔑 API 密钥配置

可通过环境变量或交互式输入设置 API 密钥：

```bash
# 智谱 GLM
export Z_AI_API_KEY="your_key"

# MiniMax
export MINIMAX_API_KEY="your_key"

# DeepSeek
export DEEPSEEK_API_KEY="your_key"

# Kimi
export KIMI_API_KEY="your_key"

# Moonshot
export MOONSHOT_API_KEY="your_key"

# KatCoder
export KAT_CODER_API_KEY="your_key"
export WANQING_ENDPOINT_ID="ep-xxx-xxx"
```

> 💡 提示：未设置环境变量时，工具会自动交互式提示输入密钥。

## 🎯 支持的 AI 提供商

| 提供商 | 模板名称 | 别名 | 特点 | 推荐度 |
|--------|----------|------|------|--------|
| 🌟 智谱 GLM | `zai` | `glm`, `zhipu` | 256K 上下文，高性价比 | ⭐⭐⭐⭐⭐ |
| 🔥 MiniMax | `minimax` | - | Anthropic 兼容，功能丰富 | ⭐⭐⭐⭐⭐ |
| 🚀 DeepSeek | `deepseek` | `ds` | 价格便宜，响应快速 | ⭐⭐⭐⭐ |
| 🌙 Kimi | `kimi` | `kimi-for-coding` | 专注编程，响应快速 | ⭐⭐⭐⭐ |
| 🌈 K2 | `k2` | `moonshot` | 大上下文，平衡性能 | ⭐⭐⭐ |
| 🧠 K2 Thinking | `k2-thinking` | - | 高速推理，256K 上下文 | ⭐⭐⭐⭐ |
| 🔧 KatCoder Pro | `kat-coder-pro` | `katpro` | 专业编程 AI | ⭐⭐⭐⭐ |
| 💨 KatCoder Air | `kat-coder-air` | `katair` | 轻量级快速响应 | ⭐⭐⭐ |

## 📁 高级用法

### 作用域控制

```bash
# 仅应用环境变量
ccs apply zai --scope env

# 仅应用常用设置（提供商、模型等）
ccs apply zai --scope common

# 应用完整配置（默认）
ccs apply zai --scope all
```

### 自定义配置

```bash
# 指定模型
ccs apply zai --model glm-4-plus

# 应用前备份当前设置
ccs apply zai --backup

# 跳过确认提示
ccs apply zai --yes

# 应用到全局配置
ccs apply zai --settings-path ~/.claude/settings.json
```

### 高级快照

```bash
# 创建仅包含环境变量的快照
ccs snap my-env --scope env

# 创建带描述的快照
ccs snap my-config --description "开发环境配置"

# 从自定义文件创建快照
ccs snap my-custom --settings-path /path/to/settings.json
```

## 📋 命令参考

| 命令 | 别名 | 说明 |
|------|------|------|
| `ccs apply <target>` | `a` | 应用模板或快照 |
| `ccs snap <name>` | `s` | 创建快照 |
| `ccs ls` | `list` | 列出快照 |
| `ccs delete <name>` | `rm/del` | 删除快照 |
| `ccs credentials <cmd>` | `creds` | 凭证管理 |

凭证管理子命令：
- `list`: 列出凭证
- `delete <id>`: 删除指定凭证
- `clear`: 清除所有凭证

## 🧩 扩展开发

### 架构概述

```
src/
├── lib.rs          # 核心 trait 定义
├── cli.rs          # CLI 解析
├── commands.rs     # 命令实现
├── settings.rs     # 配置模型
├── snapshots.rs    # 快照系统
├── credentials.rs  # 凭证管理
├── templates/      # 模块化模板
│   ├── mod.rs      # Template trait
│   ├── zai.rs      # 智谱 GLM
│   ├── minimax.rs  # MiniMax
│   └── ...         # 其他提供商
└── utils.rs        # 工具函数
```

### 添加新 AI 提供商

1. **创建模板文件**：在 `src/templates/` 下创建新文件（如 `new_provider.rs`）

2. **实现 Template trait**：
   ```rust
   use crate::{
       settings::ClaudeSettings,
       snapshots::SnapshotScope,
       templates::{Template, TemplateType},
   };

   pub struct NewProviderTemplate;

   impl Template for NewProviderTemplate {
       fn template_type(&self) -> TemplateType {
           TemplateType::NewProvider
       }

       fn display_name(&self) -> &'static str {
           "New Provider"
       }

       fn description(&self) -> &'static str {
           "A new AI provider template"
       }

       fn env_var_name(&self) -> &'static str {
           "NEW_PROVIDER_API_KEY"
       }

       fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
           let mut settings = ClaudeSettings::new();
           // 配置环境变量和设置
           settings
       }
   }
   ```

3. **注册模板**：在 `src/templates/mod.rs` 中添加：
   - 模板类型枚举
   - 字符串解析支持
   - 工厂函数注册

4. **测试**：添加单元测试到 `tests/template_tests.rs`

## 🛠️ 开发指南

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 运行特定测试
cargo test template_tests

# 检查代码质量
cargo check
cargo clippy
cargo fmt

# 生成文档
cargo doc --open
```

## 📝 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

> 💡 **小贴士**：
> - 使用 `ccs apply <template> --backup` 避免配置丢失
> - 首推使用智谱 GLM (zai)，性价比最高
> - 遇到凭证问题可使用 `ccs credentials clear` 重置

> 🔧 **故障排除**：
> - 网络问题：检查网络连接
> - 凭证错误：重置凭证并重新保存
> - 配置冲突：使用 `--settings-path` 指定配置文件路径
