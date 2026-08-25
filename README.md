<p align="center">
  <img src="public/ptools-icon.png" width="112" alt="PTools Logo" />
</p>

<h1 align="center">PTools</h1>

<p align="center">macOS 剪贴板工具</p>

<p align="center">
  <a href="README.md">简体中文</a> | <a href="README_EN.md">English</a>
</p>

<p align="center">
  <a href="https://github.com/white-super/PTools/releases">
    <img src="https://img.shields.io/github/downloads/white-super/PTools/total?style=flat-square&amp;label=Downloads" alt="GitHub Downloads" />
  </a>
</p>

<p align="center">
  <a href="#界面预览">界面预览</a> · <a href="#下载与安装">下载与安装</a> · <a href="#主要功能">主要功能</a> · <a href="#json-格式化">JSON 格式化</a> · <a href="#使用方法">使用方法</a> · <a href="CHANGELOG.md">版本历史</a> · <a href="#隐私与权限">隐私与权限</a> · <a href="#开源许可">开源许可</a>
</p>

PTools 是一个以粘贴板为载体的工具集，目标是将日常经常使用的工具都集中到最常用的复制粘贴场景中。目前包含最基础的粘贴历史，收藏标签等。PTools 所有数据都存本地，全程不联网，放心使用。

## 界面预览

### 粘贴面板

![PTools 粘贴面板](images/Panel.png)

### 设置界面

![PTools 设置界面](images/Setting.png)

## 下载与安装

1. 前往 [版本发布页](https://github.com/white-super/PTools/releases) 下载与你的 Mac 芯片匹配的 `.dmg`：
   - `aarch64`：Apple Silicon，适用于 M 系列芯片。
   - `x86_64`：Intel Mac。
2. 打开 `.dmg`，将 **PTools** 拖入“应用程序”文件夹。
3. 首次启动后，按下默认快捷键 `Ctrl + V` 呼出剪贴板面板。

### macOS 首次安装信任与权限

GitHub 下载的版本目前使用临时签名，macOS 可能提示无法验证开发者，或者辅助功能开关打开后仍无法自动粘贴。将应用拖入“应用程序”后，在“终端”中执行以下命令（不需要安装 Tauri、Node.js 或其他开发工具）：

```bash
osascript -e 'quit app "PTools"' 2>/dev/null || true
sudo xattr -dr com.apple.quarantine /Applications/PTools.app
sudo codesign --force --deep --sign - \
  --requirements '=designated => identifier "com.white.ptools"' \
  /Applications/PTools.app
tccutil reset Accessibility com.white.ptools
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
  -f /Applications/PTools.app
killall Dock
open /Applications/PTools.app
```

执行 `sudo` 命令时输入你的 macOS 登录密码（输入过程中不会显示字符）。随后打开“系统设置 → 隐私与安全性 → 辅助功能”，点击底部的 `+`，选择 `/Applications/PTools.app` 并开启 **PTools**。

授权后请完全退出 PTools（包括菜单栏或后台进程），再重新打开应用：

```bash
osascript -e 'quit app "PTools"'
open /Applications/PTools.app
```

如果仍提示无法打开，可在“系统设置 → 隐私与安全性”中点击“仍要打开”。每次替换或升级 `PTools.app` 后，可能需要重新执行上述命令并重新授权。

## 主要功能

- **本地历史记录**：保存文本、图片和文件剪贴板内容，数据仅保存在本机。
- **快速粘贴**：双击、回车或数字键选择记录，将内容恢复到系统剪贴板。
- **便捷标签**：可添加自定义标签，给粘贴板内容分类，长久保留。
- **可控保存**：可修改快捷键、记录类型、保留时间和最大历史数量，或一键清空历史。
- **JSON 格式化**：识别 JSON/JSONC 剪贴板内容，支持格式化、压缩、去注释，以及转换为 XML 或 TypeScript。

## JSON 格式化

1. 在粘贴板面板中选中 JSON 或 JSONC 文本记录。
2. 按 `F` 打开独立的 JSON 格式化窗口；带转义的 JSON 文本（例如 `{\"name\":\"value\"}`）也可以识别。
3. 使用工具栏执行重新格式化、折叠/展开、去除注释、压缩复制、压缩转义复制，或转换为 XML/TypeScript 后复制。
4. 使用 `Command + D` 或 `Alt + D` 固定/取消固定窗口；固定窗口不会因失去焦点自动关闭，也可以同时打开多个格式化窗口。

## 使用方法

1. 复制文本、图片或文件，PTools 会按当前设置在本机记录它。
2. 按 `Ctrl + V` 打开面板；可在设置中修改为自己的组合键。
3. 通过顶部分类或标签筛选内容，单击选中记录。
4. 双击记录、按回车或按数字 `1`–`5` 恢复内容。
5. 若开启“**双击后自动粘贴**”，PTools 会将内容粘贴回原应用；否则只恢复到系统剪贴板。

## 隐私与权限

PTools 不上传剪贴板历史。文本、图片、文件路径和设置均保存在你的本机；你可以随时缩短保留时间、关闭某类记录或清空全部历史。详见 [隐私说明](PRIVACY.md)。

若要使用全局快捷键和自动粘贴，请在 macOS 的“系统设置 → 隐私与安全性 → 辅助功能”中允许 **PTools**。该权限仅用于响应快捷键及在你明确选择记录后发送粘贴操作。

## 问题反馈

欢迎通过 [问题反馈](https://github.com/white-super/PTools/issues) 提交问题或功能建议。

各版本的主要功能和变化请查看 [版本历史](CHANGELOG.md)。

## 开源许可

PTools 基于 [Apache License 2.0](LICENSE) 开源。
