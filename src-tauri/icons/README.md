# 图标文件

此目录需要包含应用程序图标文件。

## 所需文件

- `icon.ico` - Windows图标文件 (32x32, 64x64, 128x128, 256x256)
- `icon.png` - 通用PNG图标 (512x512)
- `128x128.png` - 128x128像素PNG图标
- `128x128@2x.png` - 256x256像素PNG图标（高分辨率）
- `32x32.png` - 32x32像素PNG图标
- `Square30x30Logo.png` - Windows Store图标
- `Square44x44Logo.png` - Windows Store图标
- `Square71x71Logo.png` - Windows Store图标
- `Square89x89Logo.png` - Windows Store图标
- `Square107x107Logo.png` - Windows Store图标
- `Square142x142Logo.png` - Windows Store图标
- `Square150x150Logo.png` - Windows Store图标
- `Square284x284Logo.png` - Windows Store图标
- `Square310x310Logo.png` - Windows Store图标
- `StoreLogo.png` - Windows Store标志

## 临时解决方案

如果没有图标文件，可以：
1. 在tauri.conf.json中设置 `bundle.active = false` 来禁用打包
2. 或者使用在线工具生成图标文件
3. 或者从其他应用复制图标文件作为占位符

## 生成图标

可以使用以下工具生成图标：
- https://www.icoconverter.com/
- https://iconverticons.com/
- Tauri官方图标生成器