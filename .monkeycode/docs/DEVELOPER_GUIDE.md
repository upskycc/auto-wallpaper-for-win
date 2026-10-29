# 开发指南

## 环境要求

- Rust 1.74+（开发机可在 Linux 上编写与单测）
- 目标：Windows 10/11 x64
- 交叉编译可选：`x86_64-pc-windows-gnu` + MinGW
- 真机验证需要一台 Windows 与管理员权限

## 快速开始

```bash
# 运行配置与下载单测
cargo test

# 前台跑调度（Linux 上 apply 为空操作）
cargo run -- once
```

Windows 发布：

```bash
cargo build --release --target x86_64-pc-windows-msvc
```

管理员安装：

```powershell
.\target\release\wallshift.exe install
Start-Service WallShift
```

## 项目结构说明

- `src/config.rs` 配置结构与默认值
- `src/download.rs` 可在任意平台测试
- `src/apply.rs` Win32 调用，非 Windows 编译为桩
- `src/service.rs` 仅 `cfg(windows)`

## 开发规范

- 模块与类型用英文；用户文档用中文
- 不在代码中写注释，除非说明非显然的 Windows 会话约束
- 提交信息：`feat:` / `fix:` / `chore:`
- 不把密钥写入仓库；图源 URL 可以明文写在示例配置

## 常见任务

改默认图源：编辑 `src/config.rs` 中 `default_sources` 与 `config.example.toml`。

改调度粒度：`src/scheduler.rs` 中的苏醒间隔常量。

## 构建与发布

```bash
cargo test
cargo build --release --target x86_64-pc-windows-msvc
```

产物为单个 `wallshift.exe`。安装时服务 `binPath` 指向该文件的绝对路径。
