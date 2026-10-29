# 接口定义

## 命令行

| 命令 | 权限 | 行为 |
|------|------|------|
| `wallshift.exe install` | 管理员 | 注册 Automatic / LocalSystem 服务 `WallShift` |
| `wallshift.exe uninstall` | 管理员 | 停止并删除服务 |
| `wallshift.exe service` | SCM | 服务入口 |
| `wallshift.exe run` | 当前用户 | 前台调度，便于调试 |
| `wallshift.exe once` | 当前用户 | 对已启用通道各执行一轮后退出 |
| `wallshift.exe apply wallpaper <path>` | 用户会话 | 设置桌面壁纸 |
| `wallshift.exe apply lockscreen <path>` | 管理员 | 设置机器锁屏 |

全局可选：`--config <path>`。

## 配置文件

路径默认：安装目录下的 `config\config.toml`（exe 在 `bin\` 时，安装目录为 `bin` 的上一级）。

```toml
[service]
cache_dir = "cache"

[wallpaper]
enabled = true
interval_minutes = 60
selection = "random"
sources = ["https://wp.upx8.com/api.php?resolution=3840x2160"]

[lockscreen]
enabled = true
interval_minutes = 120
selection = "random"
sources = ["https://wp.upx8.com/api.php?resolution=3840x2160"]
```

`selection` 取值：`random` | `round-robin`。

`interval_minutes` 最小为 1。

## 状态文件

安装目录下的 `config\state.json` 保存各通道上次成功 Unix 时间与 round-robin 下标。

每次换图覆盖安装目录 `cache\wallpaper.*` 与 `cache\lockscreen.*`，不保留历史图。

## 下载约定

- 超时：连接 10 秒，整体 30 秒
- 体积上限：40 MiB
- 识别：文件头 magic bytes（JPEG / PNG / BMP / WEBP）
- 失败：同一轮尝试该通道下一条 URL
