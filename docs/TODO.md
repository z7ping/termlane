# XTerminal Pro — 待办清单

## 🔴 打包发布（阻塞）
- [ ] 安装系统依赖：`sudo apt install pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
- [ ] Linux 打包：`./build.sh`
- [ ] Windows 打包：在 Windows 本机执行 `cargo-tauri build`
- [ ] macOS 打包：在 Mac 本机执行 `cargo-tauri build`

## 🟡 功能增强
- [ ] 终端链接可点击（xterm WebLinksAddon）
- [ ] 命令自动补全（Tab 补全 + 模糊匹配）
- [ ] 多服务器对比视图（并排显示输出）
- [ ] 终端配色方案（Dracula/Monokai/Solarized 等）
- [ ] 快捷键自定义（用户自己设置）

## 🟢 工程优化
- [ ] 错误边界组件（防止单组件崩溃）
- [ ] 虚拟滚动（大目录性能优化）
- [ ] 懒加载（非活跃标签不渲染）
- [ ] 配置加密存储（keyring/AES）
- [ ] 启动性能优化（首屏 < 500ms）

## 🟢 遗留小项
- [ ] 拖拽排序连接/标签
- [ ] 收藏/置顶连接
- [ ] 标签页固定（钉住）
- [ ] 面板折叠/展开动画

## ✅ 已完成的用户体验优化
- [x] 新手引导（首次打开5步引导）
- [x] 操作提示气泡（Tooltip组件）
- [x] 连接速度测试（延迟柱状图+评级）
- [x] 会话录像导出 asciinema 格式
- [x] 多语言支持 i18n（中/英）
