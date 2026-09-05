# 格式化工具手动测试样本

使用方法：每次只复制一个代码块中的内容，打开 PTools 粘贴面板，选中对应记录后按 `F`。

## JSON

预期：识别为 `JSON`，打开后自动格式化；`Ctrl + R` 重新格式化，`Ctrl + Q/W/A/S/T` 分别执行去注释、压缩、转义、转 XML、转 TypeScript。

```json
{"name":"PTools","version":"1.4.0","enabled":true,"features":["clipboard","formatter"],"config":{"maxItems":200,"theme":"dark"}}
```

## JSONC

预期：识别为 `JSON`，保留注释；点击“去除注释”后生成标准 JSON。

```jsonc
{"name":"PTools","version":"1.4.0",/* 当前开发版本 */"features":["JSON","XML","HTML"],}
```

## 转义 JSON

预期：识别为 `JSON`，打开后自动还原转义内容并格式化。

```text
{\"name\":\"PTools\",\"message\":\"escaped json\",\"items\":[1,2,3]}
```

## XML

预期：识别为 `XML`，打开后按层级缩进；点击“重新格式化”或按 `Ctrl + R`，结果保持一致。

```xml
<?xml version="1.0" encoding="UTF-8"?><project><name>PTools</name><version>1.4.0</version><features><feature>clipboard</feature><feature>formatter</feature></features></project>
```

## HTML

预期：识别为 `HTML`，打开后按标签层级缩进；点击“重新格式化”或按 `Ctrl + R`，结果保持一致。

```html
<!DOCTYPE html><html><head><title>PTools</title></head><body><main><h1>Formatter Test</h1><p>Hello PTools</p></main></body></html>
```

## URL

预期：识别为 `URL`；点击按钮或按 `Ctrl + E` 编码，再按 `Ctrl + D` 解码并恢复原文。

```text
https://example.com/search?keyword=PTools格式化&version=1.4.0
```

## Base64

预期：识别为 `Base64`；点击按钮或按 `Ctrl + D` 解码后得到 `PTools Base64 测试：你好，世界！`，按 `Ctrl + E` 可再次编码。

```text
UFRvb2xzIEJhc2U2NCDmtYvor5XvvJrkvaDlpb3vvIzkuJbnlYzvvIE=
```

## 日期字符串

预期：识别为 `日期`，打开后转换为 ISO 日期；点击“转为时间戳”或按 `Ctrl + E` 得到毫秒时间戳。

```text
2026-09-02 14:30:00+08:00
```

## 10 位时间戳

预期：识别为 `日期`，打开后显示 `2026-01-01T00:00:00.000Z`。

```text
1767225600
```

## 13 位时间戳

预期：识别为 `日期`，打开后显示 `2026-01-01T00:00:00.000Z`。

```text
1767225600000
```
