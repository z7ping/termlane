# XTerminal Pro - 测试指南

本文档描述 XTerminal Pro 的测试策略、测试编写规范和运行方法。

## 测试策略

### 前端测试（Vue 3 + Vitest）

#### 单元测试
- **目标**：覆盖所有工具函数和纯组件逻辑
- **工具**：Vitest + Vue Test Utils
- **运行**：`npm run test:unit`

#### 组件测试
- **目标**：验证组件渲染、事件处理、props 传递
- **示例**：
```-javascript
// example.spec.js
import { mount } from '@vue/test-utils'
import MyComponent from '@/components/MyComponent.vue'

describe('MyComponent', () => {
  it('should render correctly', () => {
    const wrapper = mount(MyComponent, {
      props: { title: 'Test' }
    })
    expect(wrapper.text()).toContain('Test')
  })

  it('should emit event on button click', async () => {
    const wrapper = mount(MyComponent)
    await wrapper.find('button').trigger('click')
    expect(wrapper.emitted('click')).toBeTruthy()
  })
})
```

### Rust 后端测试

#### 单元测试
- **目标**：验证模块内函数逻辑
- **工具**：Rust 内置测试框架
- **运行**：`cargo test`

#### 集成测试
- **目标**：测试 Tauri 命令和 IPC 通信
- **运行**：`cargo test --test integration`

### 测试覆盖率

| 模块 | 当前目标 | 覆盖率 | 优先级 |
|------|----------|--------|--------|
| Rust 后端 | 80% | ~15% | P0 |
| 前端工具函数 | 90% | ~5% | P0 |
| 核心组件 | 70% | ~0% | P1 |
| UI 组件 | 50% | ~0% | P2 |

## TDD 工作流（RED-GREEN-REFACTOR）

### 步骤 1：RED - 编写失败的测试
```bash
# 1. 为新功能编写测试
npm run test:unit
# 测试应该失败（RED）
```

### 步骤 2：GREEN - 编写最小可行代码
```bash
# 2. 编写刚好让测试通过的代码
# 不要过度设计，只关注功能正确性
npm run test:unit
# 测试应该通过（GREEN）
```

### 步骤 3：REFACTOR - 重构优化
```bash
# 3. 重构代码，消除重复，提高可读性
# 确保测试始终通过
npm run test:unit
# 测试应该通过（REFACTOR）
```

## 测试用例模板

### 前端单元测试模板
```javascript
// utils/example.spec.js
import { describe, it, expect } from 'vitest'
import { exampleFunction } from './example.js'

describe('exampleFunction', () => {
  it('should return correct result', () => {
    const result = exampleFunction('input')
    expect(result).toBe('expected output')
  })

  it('should handle edge cases', () => {
    expect(exampleFunction(null)).toBe(null)
    expect(exampleFunction(undefined)).toBe(undefined)
  })

  it('should throw error on invalid input', () => {
    expect(() => exampleFunction(123)).toThrow()
  })
})
```

### Rust 单元测试模板
```rust
// src/my_module.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        let input = "test";
        let result = function_name(input);
        assert_eq!(result, "expected");
    }

    #[test]
    #[should_panic(expected = "expected error")]
    fn test_invalid_input() {
        function_name("invalid");
    }
}
```

## 测试清单

### P0 - 必须有测试的核心功能
- [ ] SSH 连接建立
- [ ] SSH 密钥认证
- [ ] SSH 跳板机连接
- [ ] PTY Shell 输入/输出
- [ ] SFTP 文件列表
- [ ] SFTP 文件上传/下载
- [ ] 配置文件读写
- [ ] 窗口状态持久化

### P1 - 应该有测试的重要功能
- [ ] 连接健康监控
- [ ] 速度测试
- [ ] 会话录制
- [ ] 快捷命令执行
- [ ] 批量命令执行
- [ ] 端口转发

### P2 - 可以有测试的辅助功能
- [ ] UI 交互逻辑
- [ ] 主题切换
- [ ] 错误边界
- [ ] Toast 通知
- [ ] 键盘快捷键

## 测试最佳实践

### ✅ DO
- 使用描述性的测试名称（`should return 404 when user not found`）
- 一个测试只验证一个行为
- 使用测试数据驱动多个场景
- Mock 外部依赖（SSH、文件系统等）
- 测试边界条件（空值、极大值、特殊字符）

### ❌ DON'T
- 在测试中编写业务逻辑
- 测试第三方库的功能
- 依赖外部服务真实连接
- 在测试中使用硬编码路径
- 忽略失败的测试

## 持续集成

GitHub Actions / Gitea CI 会自动运行：

```yaml
test:
  run: |
    npm install
    npm run test:unit
    cargo test --all-features
```

每次提交都会触发测试，所有测试必须通过才能合并。

## 资源

- [Vitest 文档](https://vitest.dev/)
- [Vue Test Utils](https://test-utils.vuejs.org/)
- [Rust 测试指南](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [superpowers-tdd 技能](https://github.com/openclaw/skills/superpowers-tdd)
