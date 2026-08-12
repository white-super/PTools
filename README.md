<p align="center">
  <img src="public/ptools-icon.png" width="112" alt="PTools Logo" />
</p>

<h1 align="center">PTools</h1>

<p align="center">macOS 剪贴板工具</p>

<p align="center">
  <a href="README.md">简体中文</a> | <a href="README_EN.md">English</a>
</p>

<p align="center">
  <a href="#界面预览">界面预览</a> · <a href="#下载与安装">下载与安装</a> · <a href="#主要功能">主要功能</a> · <a href="#使用方法">使用方法</a> · <a href="#隐私与权限">隐私与权限</a> · <a href="#开源许可">开源许可</a>
</p>

PTools 是一个以复制粘贴为载体的工具集，我最开始用Paste Paste收费后换了几个开源的复制粘贴工具都不满意，所以就自己做了个PTools。PTools 所有数据都存本地，全程不联网，安全有保障放心使用。

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

> 如果 macOS 阻止首次打开，请在“系统设置 → 隐私与安全性”中确认打开该应用。

## 主要功能

- **本地历史记录**：保存文本、图片和文件剪贴板内容，数据仅保存在本机。
- **快速粘贴**：双击、回车或数字键选择记录，将内容恢复到系统剪贴板。
- **便捷标签**：可添加自定义标签，给粘贴板内容分类，长久保留。
- **可控保存**：可修改快捷键、记录类型、保留时间和最大历史数量，或一键清空历史。

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

## 开源许可

PTools 基于 [Apache License 2.0](LICENSE) 开源。
