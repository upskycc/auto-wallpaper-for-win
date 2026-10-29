# Requirements Document

## Introduction

WallShift 是运行在 Windows 10/11 上的无界面系统服务。服务按配置文件中的独立间隔，从用户提供的 HTTP(S) 图片 URL 下载图像，分别更新当前交互用户的桌面壁纸与机器锁屏。

## Glossary

- **System**: WallShift Windows 服务进程及其在用户会话中启动的 apply 助手进程
- **Config File**: TOML 配置文件，默认路径为安装目录下的 `config\config.toml`
- **Wallpaper Channel**: 桌面壁纸下载与应用通道
- **Lock Screen Channel**: 锁屏下载与应用通道
- **Image Source**: 一条 HTTP 或 HTTPS URL，每次请求返回一张图片
- **Cache Directory**: 已下载图片的本地存储目录，默认安装目录下的 `cache`
- **Active Console Session**: 当前控制台登录的交互用户会话
- **Apply Helper**: 同一可执行文件以 `--apply` 参数在用户会话中启动的短时进程

## Requirements

### Requirement 1

**User Story:** AS Windows 桌面用户, I want 服务按配置自动更换桌面壁纸, so that 桌面背景定期更新且无需打开窗口

#### Acceptance Criteria

1. WHEN Wallpaper Channel 的 `enabled` 为 true 且距上次成功应用的时间达到 `interval_minutes`，the System SHALL 从该通道的 Image Source 列表中选取一条 URL 并下载图片
2. WHEN 壁纸图片下载成功，the System SHALL 在 Active Console Session 中将该图片应用为桌面壁纸
3. WHILE Wallpaper Channel 的 `enabled` 为 false，the System SHALL 保持现有桌面壁纸不变
4. IF Active Console Session 不存在，the System SHALL 将本次壁纸应用推迟到下一轮调度

### Requirement 2

**User Story:** AS Windows 桌面用户, I want 服务按独立间隔自动更换锁屏, so that 锁屏与壁纸可以使用不同图源和不同频率

#### Acceptance Criteria

1. WHEN Lock Screen Channel 的 `enabled` 为 true 且距上次成功应用的时间达到该通道的 `interval_minutes`，the System SHALL 从锁屏 Image Source 列表中选取一条 URL 并下载图片
2. WHEN 锁屏图片下载成功，the System SHALL 通过机器级锁屏策略将该图片应用为锁屏
3. WHILE Lock Screen Channel 的 `enabled` 为 false，the System SHALL 保持现有锁屏不变
4. THE Wallpaper Channel 与 Lock Screen Channel SHALL 使用各自的间隔、图源列表与上次成功时间独立调度

### Requirement 3

**User Story:** AS 管理员, I want 用配置文件管理图源和切换时间, so that 无界面也能调整行为

#### Acceptance Criteria

1. THE System SHALL 从 Config File 读取 Wallpaper Channel 与 Lock Screen Channel 的 `enabled`、`interval_minutes` 与 `sources`
2. THE `sources` 字段 SHALL 允许包含一条或多条 HTTP/HTTPS Image Source
3. WHEN Config File 的修改时间发生变化，the System SHALL 在下一轮调度开始前重新加载 Config File
4. IF Config File 缺失或解析失败，the System SHALL 继续使用上一次成功加载的配置
5. THE 默认 Config File SHALL 包含 Image Source `https://wp.upx8.com/api.php?resolution=3840x2160`

### Requirement 4

**User Story:** AS 管理员, I want 将程序安装为开机自启的 Windows 服务, so that 无需登录交互即可按计划工作

#### Acceptance Criteria

1. WHEN 管理员以提升权限执行 `wallshift.exe install`，the System SHALL 将自身注册为名为 `WallShift` 的 LocalSystem 服务，启动类型为 Automatic
2. WHEN 管理员以提升权限执行 `wallshift.exe uninstall`，the System SHALL 停止并删除 `WallShift` 服务
3. WHEN Service Control Manager 发出 stop，the System SHALL 在 10 秒内结束调度循环并退出

### Requirement 5

**User Story:** AS 管理员, I want 下载失败时服务保持可用, so that 网络抖动不会中断后续更换

#### Acceptance Criteria

1. IF 当前 Image Source 下载失败，the System SHALL 在同一轮调度中尝试该通道列表中的下一条 Image Source
2. IF 该通道全部 Image Source 在同一轮调度中失败，the System SHALL 保留上一张已成功应用的图片
3. THE System SHALL 将单次下载的响应体限制在 40 MiB 以内
4. THE System SHALL 仅接受 Content-Type 以 `image/` 开头，或 URL 路径扩展名为 `.jpg` `.jpeg` `.png` `.bmp` `.webp` 的响应

### Requirement 6

**User Story:** AS 开发者, I want 在控制台模式验证下载与调度, so that 可以在安装服务前确认配置

#### Acceptance Criteria

1. WHEN 进程以 `run` 参数启动，the System SHALL 在前台执行与服务相同的调度循环
2. WHEN 进程以 `once` 参数启动，the System SHALL 对已启用通道各执行一次下载与应用后退出
3. WHEN 进程以 `apply wallpaper <path>` 或 `apply lockscreen <path>` 启动，the System SHALL 将指定本地文件应用到对应通道后退出
