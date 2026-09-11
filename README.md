<p align="center">
  <img src="public/ptools-icon.png" width="112" alt="PTools Logo" />
</p>

<h1 align="center">PTools</h1>

<p align="center">跨平台剪贴板工具</p>

<p align="center">
  <a href="README.md">简体中文</a> | <a href="README_EN.md">English</a>
</p>

<p align="center">
  <a href="https://github.com/white-super/PTools/releases">
    <img src="https://img.shields.io/github/downloads/white-super/PTools/total?style=flat-square&amp;label=Downloads" alt="GitHub Downloads" />
  </a>
</p>

<p align="center">
  <a href="#界面预览">界面预览</a> · <a href="#下载与安装">下载与安装</a> · <a href="#主要功能">主要功能</a> · <a href="#格式化工具">格式化工具</a> · <a href="#todo">TODO</a> · <a href="#使用方法">使用方法</a> · <a href="CHANGELOG.md">版本历史</a> · <a href="#隐私与权限">隐私与权限</a> · <a href="#开源许可">开源许可</a>
</p>

PTools 是一个以粘贴板为载体的工具集，目标是将日常经常使用的工具都集中到最常用的复制粘贴场景中。目前包含最基础的粘贴历史，收藏标签等。PTools 所有数据都存本地，全程不联网，放心使用。

## 界面预览

### 粘贴面板

![PTools 粘贴面板](images/Panel.png)

### 设置界面

![PTools 设置界面](images/Setting.png)

## 下载与安装

1. 前往 [版本发布页](https://github.com/white-super/PTools/releases) 下载对应系统的安装包：
   - `aarch64`：Apple Silicon，适用于 M 系列芯片。
   - `x86_64`：Intel Mac。
   - Windows 用户下载 `x64-setup.exe` 安装包。
2. macOS：打开 `.dmg`，将 **PTools** 拖入“应用程序”文件夹。Windows：运行 `.exe` 安装程序并按提示完成安装。
3. 首次启动后，macOS 默认按 `Ctrl + V`，Windows 默认按 `Alt + V` 呼出剪贴板面板。

### macOS 首次安装信任与权限

GitHub 下载的 macOS 版本目前使用临时签名，可能提示无法验证开发者，或者辅助功能开关打开后仍无法自动粘贴。将应用拖入“应用程序”后，在“终端”中执行以下命令（不需要安装 Tauri、Node.js 或其他开发工具）：

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
- **文本格式化工具**：识别 JSON/JSONC、XML、HTML、URL、Base64 和日期内容，并打开对应的格式化或编码转换工作区。

## 格式化工具

1. 在粘贴板面板中选中文本记录。
2. 按 `F` 打开独立的格式化工具窗口；带转义的 JSON 文本也可以识别。
3. 右键点击文本卡片并展开“使用工具”，可以查看和选择全部文本工具。
4. JSON 支持格式化、折叠/展开、去除注释、压缩复制，以及转换为 XML/TypeScript；URL、Base64 和日期支持相应的编码或格式转换。
5. 所有格式化工具支持快捷键：普通格式化使用 `Ctrl + R`；JSON 去注释、压缩、转义和转换分别使用 `Ctrl + Q/W/A/S/T`；URL、Base64 和日期转换使用 `Ctrl + E/D`。
6. 工具栏提供默认开启的“自动换行”开关，可按 `Ctrl + B` 快速切换；如果选中了编辑器中的内容，格式化、编码和解码只作用于选区，否则作用于全部内容。
7. 使用 `Command + D` 或 `Alt + D` 固定/取消固定窗口；固定窗口不会因失去焦点自动关闭，也可以同时打开多个格式化窗口。

## 文本对比（1.4.0 开发版）

1. 在粘贴面板选中一张文本或文本文件卡片，按 `D` 标记为左侧对比源；选中另一张卡片再按 `D`，打开左右对比窗口。右键菜单也提供对应入口。
2. 左侧来源以 `Diff-L` 标记，当前选中的右侧候选以 `DIFF-R` 标记；标签位于卡片头部右侧且不改变卡片尺寸。再次按 `D` 或点击标签可取消，关闭粘贴面板时会清空标记；多文件卡片会先让你选择文件。
3. 点击粘贴面板右上角的“文本对比”，可打开左右均为空的对比窗口并自行粘贴内容；从该入口打开的窗口默认置顶。
4. 默认逐字比较原文。开启“格式化对比”可比较格式化后的 JSON/JSONC、XML 或 HTML，支持自动识别和手动指定类型；转义 JSON 也适用。XML/HTML 保留有意义的空白，格式化后相同不等于原文件字节相同。
5. 支持行内差异高亮、同步滚动、差异跳转、交换左右、自动换行及折叠相同区域。`Ctrl + Q/E` 跳转上一处/下一处差异，`Ctrl + R` 重新计算，`Ctrl + B` 切换自动换行。
6. 左右内容均可直接编辑，输入后即时更新差异，支持撤销/重做；输入过程中不会自动重新格式化，开启“格式化对比”时可按 `Ctrl + R` 重新格式化。左右各自支持导入 UTF-8 文本文件（包括 UTF-8 BOM）、恢复原文和复制当前内容。不支持图片、二进制或目录对比，编辑仅修改临时副本，不会自动写回文件或历史记录。
7. 对比窗口支持多开、实时跟随主题，失焦不会关闭；`Command + D` / `Alt + D` 切换置顶，未置顶时可按 `Esc` 关闭，置顶后 `Esc` 不会关闭窗口。仍可使用全局快捷键唤起粘贴面板，关闭有临时修改的窗口前会确认。
8. 完整格式化和对比在本机后台进行，计算期间可取消；直接编辑使用本地增量差异计算，保留光标和撤销记录，不上传内容。

## TODO

- [ ] **密码管理**：随机生成密码，并以加密形式保存到本地数据库。
- [x] **扩展格式化**：支持 XML、HTML、URL encode/decode、日期格式化转换和 Base64 编码/解码。
- [x] **文本对比**：双 D 选择文本或文件，支持左右差异视图和 JSON/XML/HTML 格式化对比。
- [ ] **顺序粘贴**：开启后按顺序组织多条剪贴板内容，并通过快捷键依次粘贴。
- [ ] **cURL 请求工具**：解析 cURL 文本并在独立请求工作区中发送和查看响应。
- [ ] **图片裁剪工具**：对图片剪贴板内容进行裁剪、缩放和复制或保存。

以上功能计划在 `1.4.0` 版本中开发，具体交互和快捷键可能会在实现过程中调整。

## 使用方法

1. 复制文本、图片或文件，PTools 会按当前设置在本机记录它。
2. 按 `Ctrl + V` 打开面板；可在设置中修改为自己的组合键。
3. 通过顶部分类或标签筛选内容，单击选中记录。
4. 双击记录、按回车或按数字 `1`–`5` 恢复内容。
5. 若开启“**双击后自动粘贴**”，PTools 会将内容粘贴回原应用；否则只恢复到系统剪贴板。

## 隐私与权限

PTools 不上传剪贴板历史。文本、图片、文件路径和设置均保存在你的本机；你可以随时缩短保留时间、关闭某类记录或清空全部历史。详见 [隐私说明](PRIVACY.md)。

若要在 macOS 上使用全局快捷键和自动粘贴，请在“系统设置 → 隐私与安全性 → 辅助功能”中允许 **PTools**。Windows 不需要 macOS 辅助功能权限。该权限仅用于响应快捷键及在你明确选择记录后发送粘贴操作。

## 问题反馈

欢迎通过 [问题反馈](https://github.com/white-super/PTools/issues) 提交问题或功能建议。

各版本的主要功能和变化请查看 [版本历史](CHANGELOG.md)。

## 开源许可

PTools 基于 [Apache License 2.0](LICENSE) 开源。
