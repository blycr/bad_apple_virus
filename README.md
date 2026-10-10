# Bad Apple Windows Player

用真实的 Windows 窗口播放 Bad Apple。`bad_apple_virus` 与 `bad_apple_win`
现已合并：一个项目、一份素材、一个 `bad_apple.exe`，保留两种窗口外观。

## 下载与播放

在 [GitHub Releases](https://github.com/blycr/bad_apple_virus/releases/latest)
下载并解压发布包。双击 `bad_apple.exe` 或 `Play.cmd` 即可播放；
双击 `Play-Windows.cmd` 可使用标题栏窗口模式。音频和动画已嵌入程序，
不需要另外下载视频或安装 Rust。

| 模式 | 启动方式 | 效果 |
| --- | --- | --- |
| 无边框（默认） | `bad_apple.exe` | 白色窗口拼接画面，自动提供黑色背景 |
| 标题栏窗口 | `bad_apple.exe --mode windows` | 大块带标题栏，小块保持无边框以避免越界 |

两种模式均在主显示器播放，保持 4:3 比例，支持高 DPI，不依赖桌面壁纸。
播放时使用原始 **64×48、30 fps** 矩形素材，最多创建 **155 个画面窗口**，
另加一个背景窗口。不会自动细分或生成几百个额外窗口。

## 控制

| 快捷键 | 操作 |
| --- | --- |
| **Ctrl+Alt+F8** | 同时暂停或继续画面和音频 |
| **Ctrl+Alt+F9** | 退出并释放窗口、音频和快捷键 |

快捷键无需窗口焦点，只在运行期间注册，长按不会重复切换。
若快捷键已被其他程序占用，播放器提示并退出；请先关闭另一个播放器。
标题栏模式也可点击任意窗口的关闭按钮退出整个播放器。

```powershell
.\bad_apple.exe --mode borderless
.\bad_apple.exe --mode windows
.\bad_apple.exe --mute
.\bad_apple.exe --help
.\bad_apple.exe --version
```

## 从源码构建

需要 Windows 10/11、Rust stable，以及 Visual Studio 的 C++ 构建工具和
Windows SDK（用于 MSVC 链接和图标资源）。在项目根目录运行：

```powershell
cargo build --locked --release
.\target\release\bad_apple.exe
.\target\release\bad_apple.exe --mode windows
```

项目根目录的两个 `.cmd` 入口也支持启动本地 `target\release` 中的程序。

## 维护与验证

渲染、音频时间线、快捷键和素材解析分别位于 `src/renderer.rs`、
`src/playback.rs`、`src/controls.rs` 与 `src/video.rs`。
两种模式共用同一套逻辑，不再维护两份播放器。

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo run --locked --release --example audio_probe -- 12
cargo run --locked --release --example pause_probe
```

两个音频探针需要真实音频设备，以零音量运行，不弹出播放窗口。
测试覆盖素材边界与窗口上限、分数缩放下的共享边缘、音频启动延迟、
播放停顿与故障回退、长时间暂停以及命令行模式选择。
GitHub Actions 在 Windows 上检查格式、Clippy、测试和发布构建，并生成下载包。

画面跟随实际音频播放位置；无音频设备或音频故障时，从当前位置继续静音播放。
窗口变大、重新显示或切换标题栏后立即重绘，避免连续白色区域出现黑洞。
纯移动复用已绘制表面，以控制重绘开销。
详见 [SYNC_FIX.md](SYNC_FIX.md)、[RENDER_FIX.md](RENDER_FIX.md)
和 [RELEASING.md](RELEASING.md)。

## 合并与旧项目

从 **v1.1.0** 起，只维护此仓库的 `main` 分支和 `bad_apple.exe`。
旧的 `bad_apple_win` 独立项目已废弃，其功能由默认无边框模式接替。
`rewrite` 分支和旧 Releases 仅保留历史记录，不作为当前构建入口。
此前的 `bad_apple_win.exe` 请替换为新的 `bad_apple.exe`。

## 来源

基于 [mon/bad_apple_virus](https://github.com/mon/bad_apple_virus)，
保留原项目的 MIT 许可与版权声明。仓库名称沿用原项目；程序是窗口动画播放器。
`bad apple.py` 是原始视频预处理脚本，播放与构建现有素材不需要运行它。

原项目演示：[Bad Apple rendered with Windows windows](https://www.youtube.com/watch?v=EZpZwunMzuE)。
