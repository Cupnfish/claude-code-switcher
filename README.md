# Claude Code Switcher

一个用于管理 Claude Code 设置快照和模板的 CLI 工具，让你轻松在不同的 AI 提供商之间切换。

## 🎯 为什么需要这个工具？

Claude Code 支持多个 AI 提供商，但切换模型和配置比较麻烦。这个工具让你可以：

- 🔄 **轻松切换**：在不同 AI 提供商之间一键切换
- 📦 **模板管理**：内置多个热门 AI 提供商的预设模板
- 💾 **快照功能**：保存和恢复你自己的配置组合
- 🌍 **环境隔离**：项目级和全局配置分离

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
# 应用智普 GLM 模板（强烈推荐）
ccs apply glm

# 应用 DeepSeek 模板
ccs apply deepseek

# 应用 K2Sonnet 模板
ccs apply k2sonnet

# 应用 Longcat 模板
ccs apply longcat
```

#### 2. 创建自己的快照

```bash
# 创建当前设置的快照
ccs snap my-config

# 应用快照
ccs apply my-config
```

#### 3. 管理快照

```bash
# 查看所有快照
ccs ls -v

# 删除快照
ccs delete my-config
```

## 🔑 API 密钥配置

在使用模板之前，需要先设置对应的环境变量：

```bash
# 智普 GLM（推荐）
export Z_AI_API_KEY="your_api_key_here"

# DeepSeek
export DEEPSEEK_API_KEY="your_api_key_here"

# K2Sonnet
export K2_SONNET_API_KEY="your_api_key_here"

# Longcat
export LONGCAT_API_KEY="your_api_key_here"
```

> 💡 **提示**：如果没有设置环境变量，工具会交互式地提示你输入 API 密钥。

## 🎯 推荐配置

### 🌟 智普 GLM（强烈推荐）

智普是目前最推荐的选择：
- 💰 **性价比高**：有编程套餐，便宜好用
- 🚀 **性能优秀**：响应速度快，代码生成质量高
- 📊 **上下文充足**：支持200k上下文长度

### 其他选择

- **DeepSeek**：价格便宜，但上下文长度有限（目前128k）
- **K2Sonnet**：平衡的性能和价格
- **Longcat**：特定的优化模型

## 📁 作用域说明

默认情况下，所有配置都应用到**项目级别**（`.claude/settings.json`）：

```bash
# 默认应用到项目目录
ccs apply glm

# 应用到全局配置（需要手动指定路径）
ccs apply glm --settings-path ~/.claude/settings.json
```

## 🔧 高级用法

### 自定义模型

```bash
# 使用指定模型
ccs apply glm --model glm-4-plus
ccs apply deepseek --model claude-3-5-sonnet-20241022
```

### 备份当前设置

```bash
# 应用前先备份
ccs apply glm --backup
```

### 快照管理

```bash
# 创建不同范围的快照
ccs snap my-env --scope env        # 仅环境变量
ccs snap my-full --scope all       # 所有设置
ccs snap my-common --scope common  # 常用设置（默认）

# 从自定义文件创建快照
ccs snap my-config --settings-path /path/to/settings.json
```

## 📋 可用命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `ccs ls` | `ccs list` | 列出所有快照 |
| `ccs snap <name>` | `ccs s` | 创建快照 |
| `ccs apply <target>` | `ccs a` | 应用快照或模板 |
| `ccs delete <name>` | `ccs rm, del` | 删除快照 |

## 🛠️ 开发

```bash
# 构建
cargo build

# 运行
cargo run -- <command>

# 测试
cargo test

# 发布构建
cargo build --release
```

## 📝 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

> 💡 **小贴士**：建议把常用的 API 密钥添加到 shell 配置文件中（如 `.bashrc`、`.zshrc`），这样就不需要每次都输入了。
