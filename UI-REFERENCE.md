# Interactive UI Reference

All interactive screens in Claude Code Switcher, organized by user flow.

## Quick Test Setup

```bash
# Build the tool
cargo build

# Create test snapshots (uses current project settings)
ccs snap test-snapshot-1
ccs snap test-snapshot-2
ccs snap test-snapshot-3

# Set a dummy env var to test env var detection flow
export DEEPSEEK_API_KEY=sk-test-dummy-key-12345

# Apply a template (creates credential + saves settings)
ccs apply deepseek
```

---

## 1. Snapshot Browser

**Trigger:** `ccs list`

### 1.1 Snapshot List

Crossterm full-screen Selector with filter and management shortcuts.

```
? Select a snapshot to manage (3 items)

❯ 🔍 Filter/Custom search...
  test-snapshot-1 (common)
  test-snapshot-2 (common)
  test-snapshot-3 (all)
➕ Create New...

Type to filter, Enter to search, ↑↓ to navigate, Esc to back
```

| Key | Action |
|-----|--------|
| Type chars | Filter items in real-time |
| ↑ / ↓ | Navigate list |
| PgUp / PgDown | Page navigation |
| Home / End | Jump to start/end |
| Enter | Select snapshot → goes to **1.2** |
| → | Select / forward |
| d | Delete selected snapshot directly → goes to **1.3c** |
| n | Rename selected snapshot directly → goes to **1.3b** |
| r | Refresh list |
| ← / Esc | Back / exit |
| Ctrl+C | Exit application |

When typing a filter (cursor on 🔍 row):

```
? Select a snapshot to manage (2 items)

❯ 🔍 test
  test-snapshot-1 (common)
  test-snapshot-2 (common)
➕ Create New...

Type to filter, Enter to search, ↑↓ to navigate, Esc to back
```

When cursor is on an item:

```
? Select a snapshot to manage (3 items)

  🔍 Filter/Custom search...
❯ test-snapshot-2 (common)              ← yellow highlight
  test-snapshot-3 (all)
➕ Create New...

↑↓ to navigate, Enter to select, ←→ to move cursor, d: Delete, n: Rename, r: Refresh, Esc: Back
```

### 1.2 Snapshot Action Menu

inquire::Select — shown after selecting a snapshot from **1.1**.

```
? Action for 'test-snapshot-1':
> Apply
  Rename
  Delete
  Back

↑/↓: Navigate, Enter: Select, Esc: Back
```

### 1.3a Apply Confirmation

inquire::Confirm

```
? Apply snapshot 'test-snapshot-1'? (y/N)
```

### 1.3b Rename Input

inquire::Text — pre-filled with current name.

```
? Rename snapshot:
> test-snapshot-1

Enter new name, Esc to cancel
```

### 1.3c Delete Confirmation

inquire::Confirm

```
? Delete 'test-snapshot-1' snapshot? (y/N)
```

---

## 2. Snapshot Creation

**Trigger:** Select "➕ Create New..." from Snapshot List (**1.1**)

### 2.1 Config Path

inquire::Select

```
? Select configuration to snapshot:
> Local (.claude/settings.json) - Project-specific settings
  Global (~/.claude/settings.json) - User-wide settings

↑/↓: Navigate, Enter: Select
```

### 2.2 Preview

Read-only display (no interaction).

```
📋 Current Configuration Preview:
📁 Path: .claude/settings.json

  env:
    ANTHROPIC_BASE_URL: "https://api.deepseek.com"
    DEEPSEEK_API_KEY: "sk-••••••••"
    ...

🤖 Model: deepseek-chat
🔐 Permissions: 8 rules
```

### 2.3 Name

inquire::Text

```
? Enter snapshot name:
>

A descriptive name (e.g., 'development-setup', 'production-config')
```

### 2.4 Description

inquire::Text — optional, press Enter to skip.

```
? Enter description (optional):
>

Optional description to help you remember what this snapshot is for
```

### 2.5 Scope

inquire::Select

```
? Select snapshot scope:
> common - Common settings only (model, hooks, permissions)
  env - Environment variables only
  all - All settings (common + environment)

↑/↓: Navigate, Enter: Select
```

### 2.6 Confirm

inquire::Confirm

```
📋 Snapshot Summary:
  Name: my-new-snapshot
  Path: .claude/settings.json
  Scope: common

? Create this snapshot? (y/N)
```

If name already exists:

```
? Snapshot 'test-snapshot-1' already exists. Overwrite? (y/N)
```

---

## 3. Template Application

**Trigger:** `ccs apply <template>`

Examples: `ccs apply deepseek`, `ccs apply zai`, `ccs apply kat-coder`

### 3.1 Variant Selection

inquire::Select — only shown for templates with variants when using a generic name.

Skip variant selection with specific aliases:
```bash
ccs apply zai-china       # Skip directly to China
ccs apply kat-coder-pro   # Skip directly to Pro
ccs apply k2              # Skip directly to K2
ccs apply deepseek        # No variants, skipped entirely
```

**ZAI (zai / glm / zhipu):**
```
? Select ZAI region:
> ZAI China (智谱AI)
  ZAI International

↑/↓ to navigate, enter to select, esc to cancel
```

**KatCoder (kat-coder / kat):**
```
? Select KatCoder variant:
> KatCoder Pro - High-performance coding with Claude Opus 4.6 capabilities
  KatCoder Air - Lightweight and fast coding assistance
```

**Kimi (kimi):**
```
? Select Kimi service:
> K2 - Moonshot K2 model
  K2 Thinking - K2 with extended thinking
  Kimi For Coding - Kimi coding assistant
```

**AnyRouter (anyrouter / ar):**
```
? Select AnyRouter region:
> China (fast) - Fast response with China routing
  Fallback (stable) - Stable fallback routing
```

**OpenRouter (openrouter / or):**
```
? Select OpenRouter model:
> anthropic/claude-sonnet-4-6
  anthropic/claude-opus-4-6
  google/gemini-2.5-pro
  ...
```

### 3.2 API Key Source

inquire::Select — only shown when environment variables with API keys are detected.

```
? API key source:
> Use API key from environment variable DEEPSEEK_API_KEY
  Enter a custom API key

↑/↓ to navigate, enter to select, esc to cancel
```

If env var selected:
```
✓ Using API key from environment variable DEEPSEEK_API_KEY
✓ API key saved automatically for future use.
```

If "custom" selected → goes to **3.3** or **3.4**.

### 3.3 Credential Selection

Selector — shown when saved credentials exist for the template.

```
? Select DeepSeek API key: (1 items)

❯ 🔍 Filter/Custom search...
  DeepSeek API Key (DeepSeek) - sk-t••••••••
➕ Create New...

↑↓ to navigate, Enter to select, ←→ to move cursor, d: Delete, n: Rename, r: Refresh, Esc: Back
```

### 3.4 New API Key Input

inquire::Text — shown when no saved credentials or "Create New" selected.

```
🔑 Create New API Key

  💡 Get your API key from: https://platform.deepseek.com/api_keys

? Enter your DeepSeek API key:
> sk-...

placeholder: sk-...
```

### 3.5 Apply Confirmation

Shown without `--yes` flag. Shows diff of changes.

```
Changes to be applied:
  + model: "deepseek-chat"
  + env: ANTHROPIC_BASE_URL = "https://api.deepseek.com"
  + env: ANTHROPIC_AUTH_TOKEN = "sk-..."
  ...

? Apply these changes? (y/N)
```

**Skip:** `ccs apply deepseek --yes`

---

## 4. Snapshot Application

**Trigger:** `ccs apply <snapshot-name>` (when name doesn't match any template)

```
ccs apply test-snapshot-1
```

### 4.1 Settings Comparison + Confirm

```
Current settings:
  env:
    ANTHROPIC_BASE_URL: "https://api.deepseek.com"
    ...

Snapshot settings:
  env:
    ANTHROPIC_BASE_URL: "https://open.bigmodel.cn/api/anthropic"
    ...

? Apply these settings? (y/N)
```

**Skip:** `ccs apply test-snapshot-1 --yes`

---

## 5. Credential Browser

**Trigger:** `ccs credentials list`

### 5.1 Credential List

Selector — same crossterm component as snapshot list.

```
? Select a credential to manage (1 items)

❯ 🔍 Filter/Custom search...
  DeepSeek API Key (DeepSeek) - sk-t••••••••

↑↓ to navigate, Enter to select, ←→ to move cursor, d: Delete, n: Rename, r: Refresh, Esc: Back
```

When > 5 credentials, selecting the 🔍 filter option shows a text filter:

```
? Filter credentials:
>

Type to filter credential names, Tab: Complete, Enter: Select, Esc: Cancel
```

With autocomplete dropdown showing matching names.

### 5.2 Credential Action Menu

inquire::Select — shows credential details above options.

```
? Manage Credential:

  Credential: DeepSeek API Key (DeepSeek)
  API Key: sk-t••••
  Env: DEEPSEEK_API_KEY (primary)

  Created: 2026-03-27 10:00:00 UTC
  Updated: 2026-03-27 10:00:00 UTC

> ✏️  Rename
  🗑️  Delete
  ⬅️  Back

↑↓ to move, enter to select, esc to cancel
```

### 5.3a Rename Credential

inquire::Text — pre-filled with current name.

```
? Rename 'DeepSeek API Key':
> DeepSeek API Key

Enter new name, Esc to cancel
```

After input, confirmation:

```
? Rename 'DeepSeek API Key' to 'my-new-name' (y/N)
```

### 5.3b Delete Credential

inquire::Confirm

```
? Delete 'DeepSeek API Key' credential? (y/N)
```

---

## 6. Credential Clear

**Trigger:** `ccs credentials clear`

### 6.1 Clear Confirmation

inquire::Confirm

```
? Clear all saved credentials? (y/N)
```

**Skip:** `ccs credentials clear --yes`

---

## Component Reference

| Component | Library | Usage |
|-----------|---------|-------|
| **Selector** | crossterm | Full-screen filterable list with keyboard shortcuts. Used for snapshot list, credential list, API key selection. |
| **Select** | inquire | Simple option menu. Used for action menus, variant selection, scope/path selection. |
| **Text** | inquire | Text input. Used for rename, snapshot name, API key input. |
| **Confirm** | inquire | Yes/No confirmation. Used for apply/delete/overwrite confirmations. |

### Selector Key Bindings

| Key | Action |
|-----|--------|
| ↑ / ↓ | Navigate |
| PgUp / PgDown | Page navigation |
| Home / End | Jump to start/end |
| Enter | Select / View details |
| ← | Back |
| → | Select / Forward |
| d | Delete (management mode) |
| n | Rename (management mode) |
| r | Refresh |
| Ctrl+C | Exit |
| Esc | Back |
