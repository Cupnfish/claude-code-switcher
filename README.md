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
| **一键切换** | 无需手动编辑，一条命令完成 AI 提供商切换 |
| **预设模板** | 内置 12+ 热门 AI 提供商，开箱即用 |
| **快照系统** | 保存自定义配置，随时一键恢复 |
| **安全存储** | API 密钥本地加密存储，支持多凭证管理 |
| **交互式 TUI** | 全屏交互式浏览器，支持键盘快捷键操作 |
| **环境隔离** | 项目级和全局配置独立存储，互不干扰 |
| **模块设计** | 清晰的代码架构，易于扩展新提供商 |

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
# 智谱 GLM-5.2（推荐）- Coding 能力对齐 Claude Opus 4.6，支持 1M 上下文
ccs apply zai
# 别名：glm, zhipu

# MiniMax - Anthropic 兼容，高性能 AI
ccs apply minimax

# DeepSeek V4 - 思考模式，V4 Pro/Flash 模型，1M 上下文，价格优惠
ccs apply deepseek
# 别名：ds

# Fishtrip - Anthropic 兼容网关
ccs apply fishtrip
# 别名：fish

# Kimi - 专注编程场景（支持 K2、K2 Thinking 等服务）
ccs apply kimi

# KatCoder - 支持 Pro/Air 两种规格
ccs apply kat-coder

# SeedCode - 字节跳动编程模型
ccs apply seed-code

# Duojie - 多提供商聚合
ccs apply duojie
# 别名：dj

# Zenmux - 多提供商路由
ccs apply zenmux

# AnyRouter - 智能路由配置
ccs apply anyrouter
# 别名：anyr, ar

# OpenRouter - 开放模型选择
ccs apply openrouter
# 别名：or
```

> **零打扰切换**：`ccs` 会记住你的偏好（变体、上次的 API key、默认 effort、是否署名），
> 保存到 `~/.claude/ccs-prefs.json`。首次运行会一次性询问默认 effort 与署名设置；之后
> `ccs apply zai` 直接应用，只打印一行摘要，**不再逐项询问**。
> 切换提供商时采用**按 scope 合并**：模板只替换它负责的字段（env/model/permissions 等），
> 你其余的设置（hooks、自定义权限、MCP 等）会被保留，不会丢失。

#### `apply` 常用选项

```bash
ccs apply zai                  # 零打扰应用（使用记住的 key/变体/偏好）
ccs apply zai --dry-run        # 预览将写入的设置，不实际写入
ccs apply zai --switch-key      # 强制弹出 API key 选择，忽略记住的 key
ccs apply zai-china             # 指定变体（也可用 --variant zai-china）
ccs apply zai --effort max      # 本次覆盖默认 effort
ccs apply zai --api-key sk-...  # 直接指定 key，跳过选择（别名 --key）
ccs apply zai --scope env       # 只切换 env 变量
ccs apply zai --backup          # 应用前备份当前设置
```

#### 其它命令

```bash
ccs current                     # 显示当前生效的提供商/模型/key/effort（别名 status）
ccs config                      # 查看/编辑偏好（effort、署名、scope）
ccs config --effort max         # 直接设置默认 effort
ccs config --co-author false    # 关闭署名（--co-author 则开启）
ccs ls                          # 快照浏览器（创建/应用/删除）
ccs credentials list            # 凭据浏览器（重命名/删除）
```


---

## 支持的 AI 提供商

| 提供商 | 命令 | 别名 | 特点 | 推荐度 |
|--------|--------|------|------|--------|
| **智谱 GLM** | `ccs apply zai` | `glm`, `zhipu` | GLM-5.2，1M 上下文，128K 输出，Coding 对齐 Claude Opus 4.6 | ⭐⭐⭐⭐⭐ |
| **MiniMax** | `ccs apply minimax` | - | Anthropic 兼容，支持中国区/国际区 | ⭐⭐⭐⭐ |
| **DeepSeek V4** | `ccs apply deepseek` | `ds` | 思考模式，V4 Pro/Flash 模型，1M 上下文，价格优惠 | ⭐⭐⭐⭐ |
| **OpenRouter** | `ccs apply openrouter` | `or` | 开放模型选择，支持多种模型 | ⭐⭐⭐⭐ |
| **AnyRouter** | `ccs apply anyrouter` | `anyr`, `ar` | 智能路由，支持中国区/Fallback | ⭐⭐⭐⭐ |
| **SeedCode** | `ccs apply seed-code` | `seedcode` | 字节跳动编程模型 | ⭐⭐⭐ |
| **Fishtrip** | `ccs apply fishtrip` | `fish` | Anthropic 兼容网关 | ⭐⭐⭐ |
| **Kimi** | `ccs apply kimi` | `k2`, `moonshot` | 统一 Moonshot 服务（K2, K2 Thinking, Kimi For Coding） | ⭐⭐⭐ |
| **KatCoder** | `ccs apply kat-coder` | `kat` | 支持 Pro/Air 两种规格 | ⭐⭐⭐ |
| **Duojie** | `ccs apply duojie` | `dj` | 多提供商聚合 | ⭐⭐⭐ |
| **Zenmux** | `ccs apply zenmux` | - | 多提供商路由 | ⭐⭐⭐ |
| **Longcat** | `ccs apply longcat` | - | LongCat 聊天配置 | ⭐⭐ |

---

## 命令参考

### 基本命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `ccs apply <target>` | `a` | 应用模板或快照 |
| `ccs ls` | `list`, `l` | 交互式快照浏览器（创建、应用、删除快照） |
| `ccs creds list` | `ccs creds ls` | 交互式凭证浏览器 |

### 快照管理

快照管理通过交互式 TUI 浏览器完成：

```bash
# 打开快照浏览器
ccs ls
```

在交互式界面中可以：
- 浏览所有快照
- 创建新快照
- 应用快照
- 删除快照

### 凭证管理

```bash
# 打开凭证浏览器（交互式管理）
ccs credentials list
# 或简写
ccs creds list

# 清除所有凭证
ccs credentials clear
```

---

## 高级用法

### 作用域控制

```bash
# 仅应用环境变量
ccs apply zai --scope env

# 仅应用常用设置（模型、权限等）- 默认
ccs apply zai --scope common

# 应用完整配置
ccs apply zai --scope all
```

### 其他选项

```bash
# 应用前备份当前配置
ccs apply zai --backup

# 跳过确认提示
ccs apply zai --yes

# 覆盖模型设置
ccs apply deepseek --model "claude-3-5-sonnet-20241022"

# 指定配置文件路径
ccs apply zai --settings-path ~/.claude/settings.json
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
├── cli.rs           # CLI 参数解析
├── commands.rs      # 命令实现
├── settings.rs      # 配置模型
├── snapshots.rs     # 快照系统
├── credentials.rs   # 凭证管理
├── utils.rs         # 工具函数
├── selectors/       # 交互式选择器框架
│   ├── base.rs      # 核心 trait 和实现
│   ├── confirmation.rs # 确认服务
│   ├── error.rs     # 选择器错误类型
│   ├── snapshot.rs  # 快照选择器（TUI）
│   ├── credential.rs # 凭证选择器（TUI）
│   └── template.rs  # 模板选择器
├── templates/       # AI 提供商模板
│   ├── mod.rs       # Template trait 定义与注册
│   ├── zai.rs       # 智谱 GLM
│   ├── deepseek.rs  # DeepSeek
│   ├── minimax.rs   # MiniMax
│   ├── kimi.rs      # Kimi/Moonshot
│   ├── kat_coder.rs # KatCoder
│   ├── fishtrip.rs  # Fishtrip
│   ├── longcat.rs   # Longcat
│   ├── seed_code.rs # SeedCode
│   ├── zenmux.rs    # Zenmux
│   ├── duojie.rs    # Duojie
│   ├── anyrouter.rs # AnyRouter
│   └── openrouter.rs # OpenRouter
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
