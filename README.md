# Claude Code Switcher

<div align="center">

[![CI](https://github.com/Cupnfish/claude-code-switcher/workflows/CI/badge.svg)](https://github.com/Cupnfish/claude-code-switcher/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Cupnfish/claude-code-switcher?display_name=release)](https://github.com/Cupnfish/claude-code-switcher/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

**一键切换 Claude Code AI 提供商的命令行工具**

</div>

---

## 简介

Claude Code Switcher (`ccs`) 是一个专为 Claude Code 设计的 CLI 工具，帮助你在不同的 AI 提供商之间快速切换配置。无需手动编辑配置文件，一个命令即可完成切换。

**主要解决的问题：**
- 想试试不同的 AI 提供商（智谱、DeepSeek、MiniMax 等）
- 需要在不同项目间使用不同的 AI 配置
- 频繁切换 API 密钥和模型设置太繁琐

---

## 核心特性

| 特性 | 说明 |
|------|------|
| 🔄 **一键切换** | 无需手动编辑，一条命令完成 AI 提供商切换 |
| 📦 **预设模板** | 内置 9+ 热门 AI 提供商，开箱即用 |
| 💾 **快照系统** | 保存自定义配置，随时一键恢复 |
| 🔐 **安全存储** | API 密钥本地加密存储，支持多凭证管理 |
| 🎨 **统一交互** | VSCode 风格的命令面板，操作直观流畅 |
| 🌍 **环境隔离** | 项目级和全局配置独立存储，互不干扰 |
| 🧩 **模块设计** | 清晰的代码架构，易于扩展新提供商 |

---

## 快速开始

### 安装

#### 从 crates.io 安装

```bash
cargo install claude-code-switcher
```

#### 从源码安装

```bash
git clone https://github.com/Cupnfish/claude-code-switcher.git
cd claude-code-switcher
cargo install --path .
```

#### 下载预编译二进制

访问 [Releases](https://github.com/Cupnfish/claude-code-switcher/releases) 页面，选择对应平台的二进制文件下载：

| 平台 | 文件名 |
|--------|----------|
| Linux x86_64 | `ccs-x86_64-linux` |
| Linux aarch64 | `ccs-aarch64-linux` |
| macOS x86_64 | `ccs-x86_64-macos` |
| macOS Apple Silicon | `ccs-aarch64-macos` |
| Windows | `ccs-x86_64-windows.exe` |

下载后赋予执行权限：
```bash
chmod +x ccs-*
mv ccs-x86_64-linux /usr/local/bin/ccs  # 或添加到 PATH
```

### 验证安装

```bash
ccs --version
ccs --help
```

---

## 基本使用

### 应用预设模板

```bash
# 智谱 GLM-5（推荐）- Coding 能力对齐 Claude Opus 4.5，支持 200K 上下文
ccs apply zai
# 别名：glm, zhipu

# MiniMax - Anthropic 兼容，功能丰富
ccs apply minimax

# DeepSeek - 价格优惠，响应快速
ccs apply deepseek
# 别名：ds

# Fishtrip - Anthropic 兼容网关
ccs apply fishtrip
# 别名：fish

# Kimi For Coding - 专注编程场景
ccs apply kimi

# KatCoder Pro - 专业编程 AI
ccs apply kat-coder-pro
# 别名：katpro

# KatCoder Air - 轻量级快速响应
ccs apply kat-coder-air
# 别名：katair
```

> **首次使用**：工具会提示输入 API 密钥，可选择保存到本地以便后续自动使用。

---

## 支持的 AI 提供商

| 提供商 | 命令 | 别名 | 特点 | 推荐度 |
|--------|--------|------|------|--------|
| 🌟 **智谱 GLM** | `ccs apply zai` | `glm`, `zhipu` | GLM-5，200K 上下文，128K 输出，Coding 对齐 Claude Opus 4.5 | ⭐⭐⭐⭐⭐ |
| 🔥 **MiniMax** | `ccs apply minimax` | - | Anthropic 兼容，功能丰富 | ⭐⭐⭐⭐ |
| 🚀 **DeepSeek** | `ccs apply deepseek` | `ds` | 价格优惠，响应快速 | ⭐⭐⭐⭐ |
| 🐟 **Fishtrip** | `ccs apply fishtrip` | `fish` | Anthropic 兼容网关 | ⭐⭐⭐ |
| 🎯 **Kimi** | `ccs apply kimi` | - | 专注编程，响应快速 | ⭐⭐⭐ |
| 🔧 **KatCoder Pro** | `ccs apply kat-coder-pro` | `katpro` | 专业编程 AI | ⭐⭐⭐ |
| 💨 **KatCoder Air** | `ccs apply kat-coder-air` | `katair` | 轻量级快速响应 | ⭐⭐ |

---

## 命令参考

### 基本命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `ccs apply <target>` | `a` | 应用模板或快照 |
| `ccs snap <name>` | `s` | 创建快照 |
| `ccs ls` | `list` | 列出所有快照 |
| `ccs delete <name>` | `rm`, `del` | 删除指定快照 |

### 凭证管理

```bash
# 列出所有保存的凭证
ccs credentials list
# 或简写
ccs creds list

# 删除指定凭证（通过 ID）
ccs credentials delete <credential-id>

# 清除所有凭证
ccs credentials clear
```

### 快照管理

```bash
# 创建快照
ccs snap my-debug-config

# 查看快照列表（含详情）
ccs ls -v

# 应用快照
ccs apply my-debug-config

# 删除快照
ccs delete my-debug-config
```

---

## 高级用法

### 作用域控制

```bash
# 仅应用环境变量
ccs apply zai --scope env

# 仅应用常用设置（模型、权限等）
ccs apply zai --scope common

# 应用完整配置（默认行为）
ccs apply zai --scope all
```

### 其他选项

```bash
# 应用前备份当前配置
ccs apply zai --backup

# 跳过确认提示
ccs apply zai --yes

# 指定配置文件路径
ccs apply zai --settings-path ~/.claude/settings.json
```

### 高级快照

```bash
# 指定作用域创建快照
ccs snap my-env --scope env

# 添加描述
ccs snap my-config --description "开发环境配置"

# 从自定义配置文件创建
ccs snap my-custom --settings-path /path/to/settings.json
```

---

## API 密钥配置

### 环境变量方式

```bash
# 智谱 GLM
export Z_AI_API_KEY="your_key"

# MiniMax
export MINIMAX_API_KEY="your_key"

# DeepSeek
export DEEPSEEK_API_KEY="your_key"

# Kimi
export KIMI_API_KEY="your_key"

# KatCoder
export KAT_CODER_API_KEY="your_key"
```

### 交互式输入

未设置环境变量时，工具会自动提示输入 API 密钥。

---

## 开发指南

### 构建与测试

```bash
# 构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo check
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

### 添加新 AI 提供商

1. 在 `src/templates/` 创建新模板文件
2. 实现 `Template` trait
3. 在 `src/templates/mod.rs` 注册模板类型
4. 添加测试到 `main.rs`

详见 [CLAUDE.md](CLAUDE.md) 开发文档。

---

## 项目结构

```
src/
├── main.rs          # 入口点和核心 trait
├── cli.rs          # CLI 参数解析
├── commands.rs     # 命令实现
├── settings.rs     # 配置模型
├── snapshots.rs    # 快照系统
├── credentials.rs  # 凭证管理
├── selectors/      # 统一选择器框架
│   ├── base.rs     # 核心 trait 和实现
│   ├── navigation.rs # 导航管理
│   ├── confirmation.rs # 确认对话框
│   └── ...         # 各类选择器
├── templates/      # AI 提供商模板
│   ├── mod.rs      # Template trait 定义
│   ├── zai.rs      # 智谱 GLM
│   └── ...         # 其他提供商
└── utils.rs        # 工具函数
```

---

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

---

## 贡献

欢迎提交 Issue 和 Pull Request！

---

## 常见问题

<details>
<summary>配置文件在哪里？</summary>

- **全局配置**：`~/.claude/settings.json`
- **项目配置**：`<项目目录>/.claude/settings.json`
- **快照存储**：`~/.claude/snapshots/`
</details>

<details>
<summary>如何重置所有配置？</summary>

```bash
ccs credentials clear    # 清除所有凭证
rm ~/.claude/settings.json  # 删除全局配置
```
</details>

<details>
<summary>遇到网络错误怎么办？</summary>

检查网络连接，或尝试使用代理。某些提供商可能有地区限制。
</details>

---

<div align="center">

Made with ❤️ by [Cupnfish](https://github.com/Cupnfish)

[Star](https://github.com/Cupnfish/claude-code-switcher) ⭐ if you find this helpful!

</div>
