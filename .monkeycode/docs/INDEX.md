# WallShift 文档

## 项目概述

WallShift 是一个无界面 Windows 系统服务，按 TOML 配置从 HTTP(S) 图片 URL 下载图像，独立更换桌面壁纸与锁屏。

**目标用户**: 希望开机后自动换壁纸和锁屏、不打开窗口的 Windows 10/11 用户与管理员

**核心价值**:
- 壁纸与锁屏使用各自的图源和间隔
- 以 Windows 服务常驻，开机自动运行
- Rust 实现，空闲内存低，无 GUI

## 技术选型

| 类别 | 选择 | 理由 |
|------|------|------|
| 语言 | Rust 2021 | 空闲内存约 3–8 MB，无 GC 停顿 |
| 运行形态 | Windows Service / LocalSystem | 开机自启，不依赖用户先打开程序 |
| HTTP | ureq + rustls | 同步请求，避免 tokio 运行时占用 |
| 配置 | TOML | 适合人手编辑的服务配置 |
| 壁纸 | SystemParametersInfoW | 在用户会话中设置当前桌面 |
| 锁屏 | PersonalizationCSP | 机器级锁屏，LocalSystem 可写 |

## 核心功能

### 独立调度
- **目的**: 壁纸和锁屏按不同间隔触发
- **描述**: 服务每 15 秒检查两个通道是否到期

### 网络图源
- **目的**: 仅从配置的 URL 拉图
- **描述**: 支持多条 URL，随机或轮询；默认含 `https://wp.upx8.com/api.php?resolution=3840x2160`

### 会话代理
- **目的**: 在 Session 0 服务里改到用户桌面
- **描述**: 下载完成后以当前控制台用户身份启动 `apply` 短时进程

## 文档导航

- 架构设计: 当前工作区 `/ .monkeycode/docs/ARCHITECTURE.md`
- 接口定义: 当前工作区 `/ .monkeycode/docs/INTERFACES.md`
- 开发指南: 当前工作区 `/ .monkeycode/docs/DEVELOPER_GUIDE.md`
