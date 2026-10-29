# WallShift

无界面 Windows 服务：按 TOML 配置从 HTTP(S) URL 拉图，独立更换桌面壁纸和锁屏。

默认图源：`https://wp.upx8.com/api.php?resolution=3840x2160`

## 行为

- 安装为 `WallShift` 服务（LocalSystem，开机自启）
- 壁纸和锁屏各有开关、间隔、图源列表
- 改安装目录下 `config\config.toml` 后下一轮调度自动生效
- 空闲时只做 15 秒一次的到期检查

## 下载

GitHub Actions 在每次推送到 `main` 后编译 Windows x64 包。打开仓库 **Actions → Build → 最新成功的 run → Artifacts**，下载 `WallShift-windows-x64.zip`。

打 `v*` 标签会额外发到 Releases：

```bash
git tag v0.1.0
git push origin v0.1.0
```

## 在 Windows 上构建

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release
```

产物：`target\release\wallshift.exe`

## 安装

解压 zip 后，右键 `install.cmd` → 以管理员身份运行。程序装到同目录 `bin\`，配置写到同目录 `config\config.toml`。

已打开管理员 PowerShell 时：

```powershell
Set-Location "D:\Program Files\WallShift-windows-x64"
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

卸载：管理员运行 `uninstall.cmd`。

## 配置

首次安装会写入 `config\config.toml`。示例见 `config.example.toml`。

```toml
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

`selection`：`random` 或 `round-robin`。可写多条 URL，当前 URL 失败则试下一条。

每次换图只覆盖当前壁纸/锁屏各一张本地文件（安装目录 `cache\wallpaper.*`、`cache\lockscreen.*`），不保留历史图。Windows 设置壁纸必须指向本地路径。

## 命令

| 命令 | 作用 |
|------|------|
| `wallshift.exe install` | 注册服务 |
| `wallshift.exe uninstall` | 删除服务 |
| `wallshift.exe run` | 前台调度（调试） |
| `wallshift.exe once` | 各通道立刻换一次后退出 |
| `wallshift.exe apply wallpaper <file>` | 把本地文件设为壁纸 |
| `wallshift.exe apply lockscreen <file>` | 把本地文件设为锁屏 |

锁屏写入 `PersonalizationCSP`，在 Windows 10/11 Pro 及教育/企业版上生效。

## 开发机单测

```bash
cargo test
```
