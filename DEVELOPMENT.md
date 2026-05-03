# OmniLynceus Loader - 开发文档

## 1. 项目概述

**项目名称**：OmniLynceus Loader**项目目标**：通用游戏辅助加载器 + 插件系统**核心功能**：

- 动态加载游戏识别插件
- 自动进程识别和插件激活
- 物理仿真鼠标控制（人类行为模拟）
- 实时自动瞄准/目标追踪

---

## 2. 架构设计

### 2.1 系统架构图

```
┌──────────────────────────────────────────┐
│        Tauri 前端 (UI & 控制)            │
└────────────────┬─────────────────────────┘
                 │ Tauri::invoke
┌────────────────▼─────────────────────────┐
│     加载器核心 (Rust Backend)            │
├──────────────────────────────────────────┤
│ ┌─────────────────────────────────────┐  │
│ │   插件管理器 (PluginManager)        │  │
│ ├─────────────────────────────────────┤  │
│ │ • 加载 DLL 插件                     │  │
│ │ • 进程匹配                          │  │
│ │ • 异常处理和禁用                    │  │
│ └─────────────────────────────────────┘  │
│ ┌─────────────────────────────────────┐  │
│ │   鼠标控制器 (Controller)            │  │
│ ├─────────────────────────────────────┤  │
│ │ • 144 FPS 实时控制                 │  │
│ │ • 物理仿真（质量、摩擦、抖动）      │  │
│ └─────────────────────────────────────┘  │
│ ┌─────────────────────────────────────┐  │
│ │   主循环协调                        │  │
│ ├─────────────────────────────────────┤  │
│ │ • find_active_plugin()              │  │
│ │ • call_plugin_safe()                │  │
│ │ • move_mouse()                      │  │
│ └─────────────────────────────────────┘  │
└──────────────────────────────────────────┘
         │                        │
         │ 调用 DLL 函数          │ Win32 API
         ▼                        ▼
┌──────────────────────┐    ┌──────────────┐
│ 插件 A (game1.dll)    │    │   系统接口   │
│ 插件 B (game2.dll)    │    │ • 鼠标移动   │
│ 插件 C (game3.dll)    │    │ • 截图/内存 │
└──────────────────────┘    └──────────────┘
```

### 2.2 设计原则

| 原则               | 说明                                       |
| ------------------ | ------------------------------------------ |
| **单一职责** | 加载器只管调度，插件只管识别               |
| **极简接口** | `fn find_target() -> Option<(i32, i32)>` |
| **黑盒插件** | 插件内部逻辑完全自主，加载器不干涉         |
| **高性能**   | 使用 DLL 而非 IPC，保证 144 FPS            |
| **容错能力** | panic 捕获、超时保护、异常隔离             |

---

## 3. 核心概念

### 3.1 鼠标物理模型

**参数**：

```rust
fps: 144.0              // 每秒帧数
mass: 7.8              // 质量（加速度延迟）
friction: 0.85         // 摩擦系数（阻力）
jitter_intensity: 0.004 // 随机抖动强度（人类特征）
```

**公式**：

```
加速度 = (目标距离向量 / 质量) - 摩擦(速度) + 随机抖动
速度 += 加速度
位置 += 速度
```

**特性**：

- 🎯 非直线移动，符合人类肌肉动作
- 📈 平滑加速，避免检测
- ↩️ 惯性超过目标后回调
- 🔀 随机微抖动，增加合法性

### 3.2 插件生命周期

```
1. 加载阶段
   加载器扫描 plugins/ 目录
   ↓
   打开 DLL，调用 create_plugin()
   ↓
   获得 Box<dyn VisionPlugin>

2. 激活阶段
   主循环每帧检查 is_match()
   ↓
   找到匹配的激活插件

3. 执行阶段
   调用 plugin.find_target()
   ↓
   插件内部自主：截图/内存读取/识别
   ↓
   返回 Option<(i32, i32)>

4. 控制阶段
   移动鼠标到坐标
   ↓
   应用物理模型

5. 异常阶段（可选）
   panic 捕获 → 禁用插件
   超时 → 警告 + 禁用
   None 返回 → 继续等待
```

---

## 4. 插件系统设计

### 4.1 插件 SDK 定义

**文件**：`omnilynceus-plugin/src/lib.rs`

```rust
use std::any::Any;

/// 插件元信息
#[derive(Clone, Debug)]
pub struct PluginMetadata {
    /// 唯一ID（不能重复）
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 目标进程列表 ["game1.exe", "game2.exe"]
    pub target_processes: Vec<String>,
}

/// 插件核心 Trait
pub trait VisionPlugin: Send + Sync {
    /// 返回插件元信息
    fn metadata(&self) -> PluginMetadata;
  
    /// 检查是否应该激活
    /// 通常检查：进程是否运行、窗口是否可见等
    fn is_match(&self) -> bool;
  
    /// 核心方法：返回目标坐标
    /// 内部实现完全自主（截图、内存读取、识别算法等）
    /// 返回 Some((x, y)) 或 None（未找到）
    fn find_target(&self) -> Option<(i32, i32)>;
}

/// DLL 导出函数类型
pub type CreatePluginFn = unsafe extern "C" fn() -> *mut dyn VisionPlugin;
pub type DestroyPluginFn = unsafe extern "C" fn(*mut dyn VisionPlugin);
```

### 4.2 插件开发模板

**文件**：`plugins/template_plugin.rs`（脚手架示例）

```rust
use omnilynceus_plugin::*;

pub struct MyGamePlugin {
    // 插件内部状态
}

impl VisionPlugin for MyGamePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "mygame_aimbot".to_string(),
            name: "MyGame Aimbot".to_string(),
            version: "1.0.0".to_string(),
            target_processes: vec!["mygame.exe".to_string()],
        }
    }

    fn is_match(&self) -> bool {
        // 检查进程是否运行
        // 例：使用 winapi 检查进程列表
        check_process_running("mygame.exe")
    }

    fn find_target(&self) -> Option<(i32, i32)> {
        // ===== 插件内部逻辑，完全自主 =====
      
        // 方案 A: 内存读取
        let enemy_pos = self.read_enemy_from_memory()?;
      
        // 方案 B: 截图识别
        let screenshot = self.capture_screen()?;
        let (x, y) = self.detect_enemy_by_vision(&screenshot)?;
      
        // 方案 C: 混合方案
        // ...
      
        Some((x, y))
    }
}

// 插件必须导出这两个 C 函数
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn VisionPlugin {
    Box::into_raw(Box::new(MyGamePlugin {}))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn VisionPlugin) {
    let _ = unsafe { Box::from_raw(plugin) };
}

// ===== 插件内部实现 =====
impl MyGamePlugin {
    fn read_enemy_from_memory(&self) -> Option<(i32, i32)> {
        todo!("实现内存读取逻辑")
    }
  
    fn capture_screen(&self) -> Option<Vec<u8>> {
        todo!("实现截图逻辑")
    }
  
    fn detect_enemy_by_vision(&self, screenshot: &[u8]) -> Option<(i32, i32)> {
        todo!("实现视觉识别逻辑")
    }
}
```

---

## 5. 加载器实现

### 5.1 插件管理器

**文件**：`src-tauri/src/plugin_manager.rs`

```rust
use libloading::{Library, Symbol};
use omnilynceus_plugin::{VisionPlugin, CreatePluginFn, PluginMetadata};
use std::path::Path;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub struct PluginManager {
    /// 已加载的插件
    plugins: HashMap<String, LoadedPlugin>,
    /// 禁用的插件（崩溃、超时等）
    disabled: HashSet<String>,
}

struct LoadedPlugin {
    /// DLL 句柄（用于保持引用）
    _lib: Library,
    /// 插件实例
    plugin: Arc<Box<dyn VisionPlugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            disabled: HashSet::new(),
        }
    }

    /// 从目录加载所有插件
    pub fn load_from_dir(&mut self, plugin_dir: &Path) -> Result<(), String> {
        if !plugin_dir.exists() {
            std::fs::create_dir_all(plugin_dir)
                .map_err(|e| format!("创建插件目录失败: {}", e))?;
            println!("✓ 创建插件目录: {}", plugin_dir.display());
            return Ok(());
        }

        for entry in std::fs::read_dir(plugin_dir)
            .map_err(|e| format!("读取目录失败: {}", e))?
        {
            let path = entry
                .map_err(|e| format!("读取条目失败: {}", e))?
                .path();

            // 只加载 .dll 或 .so 文件
            if !path.extension().map_or(false, |e| {
                e == "dll" || e == "so" || e == "dylib"
            }) {
                continue;
            }

            match self.load_plugin(&path) {
                Ok((id, loaded)) => {
                    let meta = loaded.plugin.metadata();
                    println!("✓ 加载插件: {} v{} [{}]", meta.name, meta.version, id);
                    self.plugins.insert(id, loaded);
                }
                Err(e) => {
                    eprintln!("✗ 加载失败 {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    /// 加载单个插件
    fn load_plugin(&self, path: &Path) -> Result<(String, LoadedPlugin), String> {
        unsafe {
            let lib = Library::new(path)
                .map_err(|e| format!("打开库失败: {}", e))?;

            let constructor: Symbol<CreatePluginFn> = lib
                .get(b"create_plugin")
                .map_err(|e| format!("找不到 create_plugin 函数: {}", e))?;

            let plugin_raw = constructor();
            let plugin = Arc::new(Box::from_raw(plugin_raw));

            let metadata = plugin.metadata();
            let id = metadata.id.clone();

            Ok((
                id,
                LoadedPlugin {
                    _lib: lib,
                    plugin,
                },
            ))
        }
    }

    /// 找到匹配的活跃插件
    pub fn find_active_plugin(&self) -> Option<Arc<Box<dyn VisionPlugin>>> {
        for (id, loaded) in &self.plugins {
            if self.disabled.contains(id) {
                continue;
            }

            if loaded.plugin.is_match() {
                let meta = loaded.plugin.metadata();
                println!("→ 激活插件: {} [{}]", meta.name, id);
                return Some(Arc::clone(&loaded.plugin));
            }
        }
        None
    }

    /// 安全调用插件
    pub fn call_plugin_safe(&mut self, plugin: &dyn VisionPlugin) -> Option<(i32, i32)> {
        let metadata = plugin.metadata();
        let plugin_id = &metadata.id;

        if self.disabled.contains(plugin_id) {
            return None;
        }

        let start = Instant::now();

        // 使用 panic 捕获隔离崩溃
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            plugin.find_target()
        }));

        let elapsed = start.elapsed();

        match result {
            Ok(Some((x, y))) => {
                // 检查超时
                if elapsed.as_millis() > 5 {
                    eprintln!(
                        "⚠ 插件 {} 处理时间过长: {}ms",
                        plugin_id,
                        elapsed.as_millis()
                    );
                }
                Some((x, y))
            }
            Ok(None) => None, // 正常未找到
            Err(_) => {
                eprintln!("✗ 插件 {} 崩溃，已禁用", plugin_id);
                self.disabled.insert(plugin_id.clone());
                None
            }
        }
    }

    /// 列出所有已加载插件
    pub fn list_plugins(&self) -> Vec<(String, PluginMetadata)> {
        self.plugins
            .iter()
            .map(|(id, p)| (id.clone(), p.plugin.metadata()))
            .collect()
    }

    /// 列出禁用的插件
    pub fn list_disabled_plugins(&self) -> Vec<String> {
        self.disabled.iter().cloned().collect()
    }
}
```

### 5.2 集成鼠标控制器

**文件**：`src-tauri/src/mouse/mod.rs`（扩展）

```rust
// ... 现有的 Controller 代码 ...

use crate::plugin_manager::PluginManager;
use std::sync::Arc;

pub struct ControllerWithPlugins {
    pub controller: Arc<Controller>,
    pub plugin_mgr: Arc<Mutex<PluginManager>>,
}

impl ControllerWithPlugins {
    pub fn new() -> Self {
        Self {
            controller: Arc::new(Controller::new()),
            plugin_mgr: Arc::new(Mutex::new(PluginManager::new())),
        }
    }

    pub fn init_plugins(&self, plugin_dir: &str) -> Result<Vec<String>, String> {
        let mut mgr = self.plugin_mgr.lock().unwrap();
        mgr.load_from_dir(std::path::Path::new(plugin_dir))?;
      
        Ok(mgr
            .list_plugins()
            .iter()
            .map(|(id, meta)| format!("{} v{} [{}]", meta.name, meta.version, id))
            .collect())
    }

    /// 启动主循环
    pub fn start_auto_aim(&self) {
        let controller = Arc::clone(&self.controller);
        let plugin_mgr = Arc::clone(&self.plugin_mgr);

        self.controller.start();

        std::thread::spawn(move || {
            loop {
                let mut mgr = plugin_mgr.lock().unwrap();

                // 1. 找活跃插件
                if let Some(plugin) = mgr.find_active_plugin() {
                    // 2. 调用插件识别
                    if let Some((x, y)) = mgr.call_plugin_safe(&**plugin) {
                        // 3. 移动鼠标
                        controller.move_to(x as f32, y as f32);
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });
    }

    pub fn stop_auto_aim(&self) {
        self.controller.pause();
    }

    pub fn list_plugins(&self) -> Vec<String> {
        let mgr = self.plugin_mgr.lock().unwrap();
        mgr.list_plugins()
            .iter()
            .map(|(id, meta)| format!("{} [{}]", meta.name, id))
            .collect()
    }
}
```

---

## 6. 项目文件结构

```
OmniLynceusLoader/
├── Cargo.workspace.toml
│
├── omnilynceus-plugin/              ← 插件 SDK
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                   (trait + 类型定义)
│
├── src-tauri/                       ← 加载器核心
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/
│       ├── main.rs                  (Tauri 入口 + Tauri commands)
│       ├── lib.rs
│       ├── mouse/
│       │   └── mod.rs               (控制器 + 插件集成)
│       └── plugin_manager.rs        (插件加载和管理)
│
├── src/                             ← 前端（Vue）
│   ├── App.vue
│   ├── main.ts
│   └── components/
│       ├── PluginList.vue
│       ├── ControlPanel.vue
│       └── Settings.vue
│
├── plugins/                         ← 运行时插件目录
│   ├── game1_aimbot.dll
│   ├── game2_aimbot.dll
│   └── template_plugin.rs           (开发模板)
│
├── vite.config.ts
├── tsconfig.json
├── package.json
└── README.md
```

---

## 7. 数据流

### 7.1 启动流程

```
用户启动应用
    ↓
Tauri 初始化
    ↓
加载器创建 (ControllerWithPlugins::new())
    ↓
前端 invoke 'init_plugins' command
    ↓
PluginManager::load_from_dir("./plugins")
    ↓
扫描 .dll 文件，逐个加载
    ↓
调用 create_plugin() 函数，获得 VisionPlugin 实例
    ↓
返回插件列表给前端
    ↓
用户点击 "启动" 按钮
    ↓
invoke 'start_auto_aim' command
    ↓
启动 Controller（鼠标控制线程）
    ↓
启动主循环线程
```

### 7.2 运行流程（每帧 ~10ms）

```
主循环
    ↓
find_active_plugin()
    ├─ 遍历所有插件
    ├─ 检查 is_match()
    └─ 返回激活的插件
    ↓
plugin.find_target()
    ├─ 插件内部逻辑（黑盒）
    ├─ 可能的内容：
    │  ├─ 内存读取
    │  ├─ 截图识别
    │  └─ 深度学习推理
    └─ 返回 Some((x, y)) 或 None
    ↓
如果返回坐标
    ├─ controller.move_to(x, y)
    ├─ 存储到 state.target
    └─ 鼠标控制线程执行物理模型
    │
    ├─ 每 1/144 秒 (6.94ms)
    ├─ 计算加速度: a = F/m - friction(v) + jitter
    ├─ 更新速度: v += a
    ├─ 更新位置: pos += v
    └─ 调用 enigo.move_mouse()
    ↓
sleep(10ms)
    ↓
重复
```

---

## 8. 开发指南

### 8.1 快速开始

**1. 克隆项目**

```bash
git clone https://github.com/OmniLynceus/OmniLynceusLoader.git
cd OmniLynceusLoader
```

**2. 编译加载器**

```bash
cd src-tauri
cargo build --release
```

**3. 创建插件项目**

```bash
cargo new --lib plugins/my_game_plugin
cd plugins/my_game_plugin
```

**4. 配置插件 Cargo.toml**

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
omnilynceus-plugin = { path = "../../omnilynceus-plugin" }
```

**5. 实现插件**

```rust
use omnilynceus_plugin::*;

pub struct MyPlugin;

impl VisionPlugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "my_game".to_string(),
            name: "My Game Aimbot".to_string(),
            version: "1.0.0".to_string(),
            target_processes: vec!["mygame.exe".to_string()],
        }
    }

    fn is_match(&self) -> bool {
        // 检查游戏是否运行
        todo!()
    }

    fn find_target(&self) -> Option<(i32, i32)> {
        // 实现目标识别
        todo!()
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn VisionPlugin {
    Box::into_raw(Box::new(MyPlugin))
}
```

**6. 编译插件**

```bash
cargo build --release
# 生成: target/release/my_game_plugin.dll
```

**7. 部署插件**

```bash
cp target/release/my_game_plugin.dll ../../plugins/
```

### 8.2 插件开发最佳实践

| 最佳实践           | 说明                                 |
| ------------------ | ------------------------------------ |
| **快速返回** | `find_target()` 应在 5ms 内完成    |
| **内存安全** | 正确处理 unsafe 代码，避免 segfault  |
| **进程检查** | 在 `is_match()` 中验证进程状态     |
| **错误处理** | 使用 `Option`，避免 panic          |
| **版本兼容** | 保持接口稳定，向后兼容               |
| **日志记录** | 使用 `println!`/`eprintln!` 调试 |

### 8.3 调试技巧

**启用日志**

```rust
// 在插件中
fn find_target(&self) -> Option<(i32, i32)> {
    eprintln!("[DEBUG] 开始识别");
    // ...
    eprintln!("[DEBUG] 找到目标: ({}, {})", x, y);
    Some((x, y))
}
```

**性能分析**

```rust
let start = std::time::Instant::now();
let result = expensive_operation();
eprintln!("耗时: {}ms", start.elapsed().as_millis());
```

---

## 9. 常见问题

**Q: 插件崩溃会影响加载器吗？**
A: 不会。使用 panic 捕获隔离，加载器会禁用崩溃的插件但继续运行。

**Q: 如何处理多个匹配的插件？**
A: 使用第一个匹配的。可在前端添加插件优先级选择。

**Q: 支持热加载吗？**
A: 当前不支持。需要修改设计以支持运行时重新加载。

**Q: 可以用 Rust 以外的语言写插件吗？**
A: 可以。只要编译为 cdylib，导出 `create_plugin` 函数即可。C++ 示例见 Wiki。

---

## 10. 相关参考

- [libloading 文档](https://docs.rs/libloading/latest/libloading/)
- [enigo 文档](https://docs.rs/enigo/latest/enigo/)
- [Tauri 文档](https://tauri.app/docs/)
- [Rust FFI 指南](https://doc.rust-lang.org/nomicon/ffi.html)

---

**文档版本**：1.0
**最后更新**：2026年5月2日
**维护者**：OmniLynceus Team
