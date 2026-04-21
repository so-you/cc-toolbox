# cc-toolbox BUG 处理清单

> **处理原则**：先记录问题（现象、根因），再解决，最后补录方案。本清单按发现时间倒序排列。

---

## BUG-004: Node.js 安装在工具私有目录，其他程序无法使用

**发现时间**：2026-04-21
**测试环境**：Windows（用户实测）

### 现象
一键安装成功下载并解压了 Node.js，但环境检测仍显示"未安装"，且安装完成后命令行中无法使用 `node`、`npm`、`claude` 命令。用户反馈："Nodejs 作为系统常用环境包，不应该装在你这个工具的文件夹下面，而应该当做系统工具下载安装。"

### 根因
Node.js 被解压到工具私有目录 `~/.cc-toolbox/nodejs/`，该目录不在系统 PATH 环境变量中。因此：
1. 其他终端/进程无法找到 `node`、`npm`
2. `system_check()` 只检查 PATH，检测不到本地 Node.js
3. npm 全局安装的 `claude` 也不在 PATH 中

### 解决方案
**安装位置改为用户级标准目录**：
- **Windows**: `%LOCALAPPDATA%\Programs\nodejs\`（用户程序标准目录）
- **macOS**: `~/.local/node/`（XDG 规范用户目录）

**安装后自动配置 PATH**：
- Windows: 通过 PowerShell `[Environment]::SetEnvironmentVariable('Path', ..., 'User')` 将 Node.js 目录写入用户 PATH 注册表
- macOS: 在 `~/.zshrc` 或 `~/.bashrc` 中追加 `export PATH="~/.local/node/bin:$PATH"`
- 安装完成后立即调用 `update_current_process_path()`，更新当前 Tauri 进程的 PATH 环境变量，使当前会话即可检测

**相关文件**：
- `src-tauri/src/installer.rs`: `nodejs_dir()`、`download_and_install_nodejs()`、`update_current_process_path()`、`add_to_windows_path()`、`add_to_macos_path()`
- `src-tauri/src/commands.rs`: `check_claude_code()`（更新本地 claude 路径）

---

## BUG-003: system_check 检测不到本地安装的 Node.js / npm / Claude Code

**发现时间**：2026-04-21
**测试环境**：Windows（用户实测）

### 现象
一键安装流程中，Node.js 下载并解压成功后，界面环境检测进度仍然显示 Node.js 和 NPM "未安装"。Claude Code 安装完成后同样检测不到。

### 根因
`commands.rs` 中的 `system_check()` 对 node、npm、claude 的检测均使用 `Command::new("xxx")` 直接查询系统 PATH。本地安装的 Node.js 不在 PATH 中（见 BUG-004），所以检测全部失败。

### 解决方案
重写 `system_check()` 检测逻辑，引入三层检测策略：
1. **本地安装位置优先**：通过 `installer::find_nodejs_bin()` 检查标准用户目录（Windows `%LOCALAPPDATA%\Programs\nodejs\`，macOS `~/.local/node/bin/`）
2. **系统 PATH 回退**：如果本地未找到，再调用 `Command::new("node")` 等检查系统 PATH
3. **Claude 额外检查**：对于 `claude`，在 PATH 检查失败后，再检查本地 Node.js 目录下的 `claude.cmd`（Windows）或 `claude`（macOS）

新增辅助函数：
- `check_nodejs()`: 本地优先 → PATH 回退
- `check_npm()`: 本地优先 → PATH 回退，npm 版本检查通过 `npm_command()` 执行（兼容 Windows `.cmd` 问题，见 BUG-002）
- `check_claude_code()`: PATH 优先 → 本地回退，Windows 本地检查使用 `cmd /C claude.cmd --version` 避免 `CreateProcessW` 限制

**相关文件**：
- `src-tauri/src/commands.rs`: `check_nodejs()`、`check_npm()`、`check_claude_code()`、`system_check()`
- `src-tauri/src/installer.rs`: `find_nodejs_bin()`（改为 `pub` 供 commands.rs 调用）

---

## BUG-002: Windows 上 npm.cmd 无法通过 Command::new 直接执行

**发现时间**：2026-04-21
**测试环境**：Windows（用户实测）

### 现象
点击一键安装后，npm 启动失败，Claude Code 安装报错。用户日志显示 npm 命令无法启动。

### 根因
Windows API `CreateProcessW`（Rust `std::process::Command` 底层）**不能直接执行完整路径的 `.cmd` 或 `.bat` 文件**。当 `npm_path` 是绝对路径如 `C:\Users\xxx\.cc-toolbox\nodejs\npm.cmd` 时，`Command::new(&npm_path)` 会启动失败。

> 注：如果通过 PATH 查找（如 `Command::new("npm")`），Windows 的 `SearchPathW` + `PATHEXT` 机制会自动调用 `cmd.exe` 解释 `.cmd` 文件，所以能工作。但完整路径直接执行不行。

### 解决方案
新增 `npm_command()` 包装函数，根据 Node.js 安装类型选择执行方式：
- **本地安装**（路径包含 `Programs\nodejs` 或 `.local/node`）：使用 `node.exe` 直接执行 `npm-cli.js`
  ```
  node.exe C:\...\npm-cli.js install -g @anthropic-ai/claude-code
  ```
- **系统 PATH 安装**：直接执行 `npm`（通过 PATH 查找，Windows 自动处理 `.cmd`）

同时新增 `npm_cli_js_path()` 函数，在已知 Node.js 安装位置中搜索 `npm-cli.js`：
- Windows: `%LOCALAPPDATA%\Programs\nodejs\node_modules\npm\bin\npm-cli.js`
- macOS: `~/.local/node/lib/node_modules/npm/bin/npm-cli.js`

**相关文件**：
- `src-tauri/src/installer.rs`: `npm_command()`、`npm_cli_js_path()`

---

## BUG-001: 一键安装按钮在环境未就绪时被禁用

**发现时间**：2026-04-21
**测试环境**：Windows（用户实测）

### 现象
Windows 全新环境（无 Node.js、无 npm）下打开应用，环境检测显示 Node.js 和 NPM "未安装"，"一键开始安装"按钮变灰不可点击。用户反馈："一键安装就要完成所有的安装，而不是无法点击。"

### 根因
`InstallPage.tsx` 中定义了 `allReady = status?.node?.installed && status?.git?.installed && status?.npm?.installed`，按钮设置了 `disabled={!allReady || installing}`。当环境不满足时，按钮直接被禁用，小白用户无法继续。

### 解决方案
- 移除 `allReady` 变量
- 按钮仅保留 `disabled={installing}`，始终可点击
- 更新按钮下方说明文字："一键自动检测环境、安装 Node.js（如缺失），并通过 npm 全局安装 Claude Code。"
- 后端 `installer.rs` 的 `install_claude_code()` 中已经实现了自动检测 → 下载安装 Node.js → 安装 Claude Code 的完整流程

**相关文件**：
- `src/pages/InstallPage.tsx`: 移除 `allReady`，修改 `disabled` 和说明文字

---

## 附：CI 配置修复

### 问题
GitHub Actions 构建报错：`Unable to resolve action dtolnay/rust-action, repository not found`

### 根因
`.github/workflows/build.yml` 中使用了错误的 action 名称 `dtolnay/rust-action@stable`，正确的名称是 `dtolnay/rust-toolchain@stable`。

### 解决方案
将 `uses: dtolnay/rust-action@stable` 改为 `uses: dtolnay/rust-toolchain@stable`。

**相关文件**：`.github/workflows/build.yml`
