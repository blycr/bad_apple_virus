# Bad Apple Windows Player v1.1.0

`bad_apple_virus` 与 `bad_apple_win` 已合并。现在只需一个 `bad_apple.exe`：

- 默认无边框模式，自带黑色背景；`--mode windows` 保留标题栏窗口外观。
- 两个模式共用渲染、音频同步、快捷键和素材解析代码。
- 保持原始 64×48、30 fps、最多 155 个画面窗口，撤回导致卡顿的额外细分。
- 修复连续白色区域中的黑洞与缝隙；小块避免标题栏最小尺寸造成的越界。
- 画面跟随实际音频位置，处理音频启动延迟、故障回退与长时间暂停。
- **Ctrl+Alt+F8** 暂停／继续音画；**Ctrl+Alt+F9** 退出。
- 增加静音选项、帮助、版本查询、两种模式启动脚本和 Windows 自动构建。
- 退出与启动失败时清理窗口、音频、计时器和全局快捷键。

下载 ZIP 后解压，双击 `bad_apple.exe` / `Play.cmd` 播放，或使用
`Play-Windows.cmd` 切换标题栏模式。音频与动画已经嵌入，无需额外素材。
独立的 `bad_apple_win` 项目及 `rewrite` 分支不再作为维护入口，历史版本保留。

发布附件中的 `SHA256SUMS` 包含 EXE 和 ZIP 的 SHA-256。
Windows 可用 `Get-FileHash .\bad_apple.exe -Algorithm SHA256` 验证；
Linux/macOS 可在两个附件齐全时使用 `sha256sum -c SHA256SUMS`。
