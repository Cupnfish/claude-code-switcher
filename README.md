# Claude Code Switcher

一个用于管理 Claude Code 设置快照和模板的 CLI 工具，让你轻松在不同的 AI 提供商之间切换。

## 🎯 为什么需要这个工具？

Claude Code 支持多个 AI 提供商，但切换模型和配置比较麻烦。这个工具让你可以：

- 🔄 **轻松切换**：在不同 AI 提供商之间一键切换
- 📦 **模板管理**：内置多个热门 AI 提供商的预设模板
- 💾 **快照功能**：保存和恢复你自己的配置组合
- 🌍 **环境隔离**：项目级和全局配置分离
- 🔐 **安全存储**：加密保存 API 密钥，支持多凭证管理

## 🚀 快速开始

### 安装

#### 方法一：从源码安装
```bash
git clone https://github.com/Cupnfish/claude-code-switcher.git
cd claude-code-switcher
cargo install --path .
```

#### 方法二：从 crates.io 安装
```bash
cargo install claude-code-switcher
```

安装完成后，你就可以使用 `ccs` 命令了。

### 基本使用

#### 1. 应用预设模板（推荐）

```bash
# 智谱 GLM（强烈推荐）
ccs apply zai
# 或使用别名
ccs apply glm
ccs apply zhipu

# MiniMax（推荐）
ccs apply minimax

# DeepSeek
ccs apply deepseek
# 或使用别名
ccs apply ds

# Kimi For Coding
ccs apply kimi

# Moonshot K2
ccs apply k2
# 或使用别名
ccs apply moonshot

# Moonshot K2 Thinking（高性能）
ccs apply k2-thinking

# 万擎 KAT-Coder Pro
ccs apply kat-coder-pro
# 或使用别名
ccs apply katpro

# 万擎 KAT-Coder Air
ccs apply kat-coder-air
# 或使用别名
ccs apply katair

# 向后兼容 - 旧的命令仍然有效
ccs apply kat-coder    # 指向 Pro 版本
ccs apply kat           # 指向 Pro 版本

# Longcat
ccs apply longcat
```

**关于凭证存储**：当使用模板时，如果环境变量未设置，工具会提示你输入 API 密钥。你可以选择将凭证保存到加密的本地存储中，下次使用时会自动提示你是否使用已保存的凭证。

#### 2. 管理保存的凭证

```bash
# 列出所有保存的凭证
ccs credentials list
# 或使用简写
ccs creds list

# 删除某个凭证（使用 ID）
ccs credentials delete <credential-id>

# 清除所有凭证（用于解决加密格式不兼容问题）
ccs credentials clear
```

保存的凭证使用 AES-256-GCM 加密存储在本地，每个凭证包含：
- API 密钥（加密存储）
- Endpoint ID（仅 KAT-Coder 需要）
- 创建时间和最后使用时间
- 可选的凭证名称

#### 3. 创建自己的快照

```bash
# 创建当前设置的快照
ccs snap my-config

# 应用快照
ccs apply my-config
```

#### 4. 管理快照

```bash
# 查看所有快照
ccs ls -v

# 删除快照
ccs delete my-config
```

## 🔑 API 密钥配置

在使用模板之前，需要先设置对应的环境变量：

```bash
# 智谱 GLM（强烈推荐）
export Z_AI_API_KEY="your_api_key_here"

# MiniMax
export MINIMAX_API_KEY="your_api_key_here"

# DeepSeek
export DEEPSEEK_API_KEY="your_api_key_here"

# Kimi For Coding
export KIMI_API_KEY="your_api_key_here"

# Moonshot K2
export MOONSHOT_API_KEY="your_api_key_here"

# Longcat
export LONGCAT_API_KEY="your_api_key_here"

# 万擎 KAT-Coder
export KAT_CODER_API_KEY="your_api_key_here"
export WANQING_ENDPOINT_ID="your_endpoint_id_here"  # 格式: ep-xxx-xxx
```

> 💡 **提示**：如果没有设置环境变量，工具会交互式地提示你输入 API 密钥。

## 🎯 支持的 AI 提供商

| 提供商 | 模板名称 | 别名 | 特点 | 推荐度 |
|--------|----------|------|------|--------|
| 🌟 **智谱 GLM** | `zai` | `glm`, `zhipu` | 高性价比，256K上下文，思考能力 | ⭐⭐⭐⭐⭐ |
| 🔥 **MiniMax** | `minimax` | `minimax-anthropic` | 高性能，Anthropic兼容，功能丰富 | ⭐⭐⭐⭐⭐ |
| 🚀 **DeepSeek** | `deepseek` | `ds` | 价格便宜，响应快速 | ⭐⭐⭐⭐ |
| 🌙 **Kimi** | `kimi` | `kimi-for-coding` | 专注编程，响应速度快 | ⭐⭐⭐⭐ |
| 🌈 **K2** | `k2` | `moonshot` | 大上下文，平衡性能 | ⭐⭐⭐ |
| 🧠 **K2 Thinking** | `k2-thinking` | `k2thinking` | 高速推理，256K上下文 | ⭐⭐⭐⭐ |
| 🔧 **KAT-Coder Pro** | `kat-coder-pro` | `katpro` | 专业编程AI，高级功能 | ⭐⭐⭐⭐ |
| 💨 **KAT-Coder Air** | `kat-coder-air` | `katair` | 轻量级，快速响应 | ⭐⭐⭐ |
| 🐱 **Longcat** | `longcat` | - | 快速高效对话AI | ⭐⭐⭐ |

### 🌟 智谱 GLM（强烈推荐）

**为什么推荐智谱 GLM？**
- 💰 **性价比极高**：提供编程专用套餐，价格合理
- 🚀 **性能优秀**：响应速度快，代码生成质量高
- 📊 **超大上下文**：支持 32000 思考令牌，256K 总上下文
- 🧠 **思考能力**：支持深度推理，适合复杂问题
- 📝 **丰富功能**：支持流式输出、工具调用等

### 🔥 MiniMax（推荐）

**MiniMax 的优势：**
- 💰 **价格合理**：有竞争力的定价策略
- 🚀 **性能出色**：支持流式输出和函数调用
- 🔄 **API 兼容**：同时支持 Anthropic 和 OpenAI 格式
- 🔧 **功能完整**：支持工具调用、并发处理等高级特性

### 🔧 万擎 KAT-Coder

万擎提供两个不同的模型版本：

**KAT-Coder Pro（推荐）**
- 🎯 **专业级**：针对复杂编程任务优化
- 💰 **按需计费**：基于实际使用量，适合专业开发
- ⚡ **完整功能**：支持所有 Claude Code 高级功能

**KAT-Coder Air**
- 🚀 **高性价比**：经济型选择，适合日常编程
- ⚡ **快速响应**：轻量级模型，响应速度更快
- 🎯 **核心功能**：支持基础的代码生成和编辑

**配置说明**：
```bash
# 设置 API 密钥
export KAT_CODER_API_KEY="your_api_key"

# 设置端点 ID（格式：ep-xxx-xxx）
export WANQING_ENDPOINT_ID="ep-12345-abcdef"

# 应用不同版本
ccs apply kat-coder-pro   # Pro 版本
ccs apply kat-coder-air   # Air 版本
ccs apply kat-coder       # 向后兼容，等同于 Pro 版本
```

## 📁 作用域说明

模板支持三种配置作用域：

```bash
# 环境变量（仅环境变量）
ccs apply zai --scope env

# 常用设置（默认：提供者、模型、端点等）
ccs apply zai --scope common

# 完整设置（环境变量 + 常用设置）
ccs apply zai --scope all
```

默认情况下，所有配置都应用到**项目级别**（`.claude/settings.json`）：

```bash
# 默认应用到项目目录
ccs apply zai

# 应用到全局配置（需要手动指定路径）
ccs apply zai --settings-path ~/.claude/settings.json
```

## 🔧 高级用法

### 自定义模型

```bash
# 使用指定模型
ccs apply zai --model glm-4-plus
ccs apply deepseek --model claude-3-5-sonnet-20241022
```

### 备份当前设置

```bash
# 应用前先备份
ccs apply zai --backup
```

### 快照管理

```bash
# 创建不同范围的快照
ccs snap my-env --scope env        # 仅环境变量
ccs snap my-full --scope all       # 所有设置
ccs snap my-common --scope common  # 常用设置（默认）

# 从自定义文件创建快照
ccs snap my-config --settings-path /path/to/settings.json

# 带描述的快照
ccs snap my-config --description "我的开发配置"
```

### 跳过确认提示

```bash
# 自动应用，不询问确认
ccs apply zai --yes
```

## 📋 可用命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `ccs ls` | `ccs list` | 列出所有快照 |
| `ccs snap <name>` | `ccs s` | 创建快照 |
| `ccs apply <target>` | `ccs a` | 应用快照或模板 |
| `ccs delete <name>` | `ccs rm, del` | 删除快照 |
| `ccs credentials <cmd>` | `ccs creds <cmd>` | 管理保存的凭证 |

凭证管理子命令：
- `ccs credentials list` - 列出所有保存的凭证
- `ccs credentials delete <id> [--yes]` - 删除指定凭证
- `ccs credentials clear [--yes]` - 清除所有凭证

## 🏗️ 架构特点

### 模块化模板系统 🧩

本项目采用全新的模块化架构：

```
src/templates/
├── mod.rs              # 主模块和 trait 定义
├── deepseek.rs         # DeepSeek 模板实现
├── zai.rs             # 智谱 GLM 模板实现
├── k2.rs              # K2 和 K2Thinking 实现
├── kat_coder.rs       # KatCoder Pro 和 Air 实现
├── kimi.rs            # Kimi 模板实现
├── longcat.rs         # Longcat 模板实现
└── minimax.rs         # MiniMax 模板实现
```

#### 🎯 Trait-based 设计

每个模板都实现了 `Template` trait：
```rust
pub trait Template {
    fn template_type(&self) -> TemplateType;
    fn display_name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn env_var_name(&self) -> &'static str;
    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings;
    fn requires_additional_config(&self) -> bool { false }
    fn get_additional_config(&self) -> Result<HashMap<String, String>> { Ok(HashMap::new()) }
}
```

#### ✨ 优势

- 🧩 **模块化**：每个提供商独立模块，易于维护
- 🔧 **可扩展**：新增提供商只需实现 trait，无需修改现有代码
- 🎯 **类型安全**：编译时保证，防止运行时错误
- 🔄 **一致性**：统一接口，标准化配置
- 📝 **丰富元数据**：内置显示名称、描述和配置提示
- ⚙️ **灵活性**：支持复杂配置需求和额外配置
- 🔙 **向后兼容**：保留所有原有功能和别名

## 🛠️ 开发

```bash
# 构建
cargo build

# 运行
cargo run -- <command>

# 测试
cargo test

# 运行模板系统演示
cargo run --example template_system

# 发布构建
cargo build --release
```

## 📚 文档

- **[TEMPLATE_SYSTEM.md](TEMPLATE_SYSTEM.md)** - 详细的模板系统架构文档
- **[examples/template_system.rs](examples/template_system.rs)** - 模板系统使用示例

## 🧪 测试

运行模板系统测试：
```bash
cargo test template_tests
```

运行所有测试：
```bash
cargo test
```

## 📝 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 添加新的 AI 提供商

要添加新的 AI 提供商：

1. 在 `src/templates/` 创建新模块
2. 实现 `Template` trait
3. 更新 `TemplateType` 枚举
4. 更新 `FromStr` 和 `Display` 实现
5. 在 `mod.rs` 中注册新模板

详细步骤请参考 [TEMPLATE_SYSTEM.md](TEMPLATE_SYSTEM.md)。

---

> 💡 **小贴士**：
> - 建议把常用的 API 密钥添加到 shell 配置文件中（如 `.bashrc`、`.zshrc`）
> - 使用 `ccs credentials list` 查看已保存的凭证
> - 使用 `--backup` 选项在应用新配置前备份当前设置
> - 智谱 GLM 是目前性价比最高的选择，推荐优先使用

> 🔧 **故障排除**：
> - 如果遇到凭证解密错误，运行 `ccs credentials clear --yes` 清除并重新保存
> - 确保网络连接正常，某些模板需要访问 API 端点
> - 检查环境变量设置是否正确