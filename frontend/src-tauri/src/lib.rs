// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::Command;
use std::thread;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // ── 启动 Python 后端（桌面，非 iOS） ──────────────────────────
            #[cfg(not(target_os = "ios"))]
            {
                let handle = app.handle().clone();
                thread::spawn(move || {
                    launch_backend(handle);
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ─────────────────────────────────────────────────────────────────────────────
// 后端启动逻辑
// ─────────────────────────────────────────────────────────────────────────────

/// 在后台线程中启动 Python 后端
#[cfg(not(target_os = "ios"))]
fn launch_backend(_app: tauri::AppHandle) {
    let bundle_root = resolve_bundle_root();
    let data_dir = resolve_data_dir();
    let data_dir_str = data_dir.to_string_lossy().into_owned();
    let bundle_root_str = bundle_root
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let port = find_free_port(8005, 20);

    #[cfg(target_os = "macos")]
    launch_mac_backend_impl(&bundle_root_str, &data_dir_str, port);

    #[cfg(target_os = "windows")]
    launch_windows_backend_impl(&bundle_root_str, &data_dir_str, port);

    #[cfg(target_os = "linux")]
    launch_linux_backend_impl(&bundle_root_str, &data_dir_str, port);
}

// ─────────────────────────────────────────────────────────────────────────────

/// 解析 bundle/app 根目录
fn resolve_bundle_root() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;

    #[cfg(target_os = "macos")]
    {
        // macOS: PlotPilot.app/Contents/MacOS/plotpilot
        // 向上: MacOS/ → Contents/ → PlotPilot.app/
        let macos_dir = exe_dir; // Contents/MacOS/
        if let Some(contents_dir) = macos_dir.parent() {
            if let Some(app_bundle) = contents_dir.parent() {
                if app_bundle
                    .file_name()
                    .map(|n| n.to_string_lossy().ends_with(".app"))
                    .unwrap_or(false)
                {
                    return Some(app_bundle.to_path_buf());
                }
            }
        }
        // Cargo dev / 非 bundle: 向上搜索项目根（含 requirements.txt）
        return search_proj_root_upward(exe_dir);
    }

    #[cfg(not(target_os = "macos"))]
    {
        return search_proj_root_upward(exe_dir);
    }
}

/// 从 start_dir 向上搜索含 requirements.txt 的目录（项目根）
fn search_proj_root_upward(start_dir: &std::path::Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        if dir.join("requirements.txt").exists() || dir.join("pyproject.toml").exists() {
            return Some(dir.clone());
        }
        if !dir.pop() {
            break;
        }
    }
    Some(start_dir.to_path_buf())
}

// ─────────────────────────────────────────────────────────────────────────────

/// 检测 CPU 架构，返回 "aarch64" 或 "x86_64"
#[cfg(target_os = "macos")]
fn detect_mac_arch() -> String {
    let arch = std::process::Command::new("uname")
        .arg("-m")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if arch == "arm64" {
        "aarch64".to_string()
    } else {
        "x86_64".to_string()
    }
}

/// 在 macOS bundle 中查找 Python 可执行文件
/// 优先级: 1) bundle 内嵌 Python, 2) 系统 python3
#[cfg(target_os = "macos")]
fn find_mac_python(resource_dir: &PathBuf) -> String {
    let arch = detect_mac_arch();

    // 1) 尝试 bundle 内嵌 Python
    let embedded_py = resource_dir
        .join("python_macos")
        .join(&arch)
        .join("bin")
        .join("python3");

    if embedded_py.exists() {
        // 验证 uvicorn 可用
        let check = std::process::Command::new(&embedded_py)
            .args(["-c", "import uvicorn"])
            .output();
        if check.is_ok() {
            println!("[PlotPilot] Using bundled Python: {:?}", embedded_py);
            return embedded_py.to_string_lossy().into_owned();
        }
    }

    // 2) 降级到系统 python3
    println!("[PlotPilot] Falling back to system python3");
    "python3".to_string()
}

#[cfg(target_os = "macos")]
fn launch_mac_backend_impl(bundle_root: &str, data_dir: &str, port: u16) {
    // macOS bundle 结构: PlotPilot.app/Contents/Resources/
    // Dev 模式: bundle_root 是项目根目录，resource_dir 就是项目根
    let resource_dir = if !bundle_root.is_empty() {
        let res_path = PathBuf::from(bundle_root).join("Contents/Resources");
        if res_path.exists() {
            // 真正的 .app bundle
            res_path
        } else {
            // Dev 模式: bundle_root 就是项目根
            PathBuf::from(bundle_root)
        }
    } else {
        // Fallback: 当前工作目录
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    // 查找 Python 可执行文件
    let python_exe = find_mac_python(&resource_dir);

    let mut cmd = Command::new(&python_exe);
    cmd.arg("-m");
    cmd.arg("uvicorn");
    cmd.arg("interfaces.main:app");
    cmd.arg("--host");
    cmd.arg("127.0.0.1");
    cmd.arg("--port");
    cmd.arg(port.to_string());
    cmd.env("AITEXT_PROD_DATA_DIR", data_dir);
    cmd.env("PYTHONPATH", resource_dir.to_string_lossy().as_ref());
    cmd.env("PYTHONIOENCODING", "utf-8");
    cmd.env("PYTHONUNBUFFERED", "1");
    cmd.env("HF_HUB_OFFLINE", "1");
    cmd.env("TRANSFORMERS_OFFLINE", "1");
    let _ = cmd.spawn();
    println!("[PlotPilot] Backend started with python3");
}

#[cfg(target_os = "windows")]
fn launch_windows_backend_impl(bundle_root: &str, data_dir: &str, port: u16) {
    let python_exe = std::env::var("PYTHON_EXE")
        .or_else(|_| std::env::var("VIRTUAL_ENV_PYTHON"))
        .unwrap_or_else(|_| "python".to_string());

    let mut cmd = Command::new(&python_exe);
    cmd.arg("-m");
    cmd.arg("uvicorn");
    cmd.arg("interfaces.main:app");
    cmd.arg("--host");
    cmd.arg("127.0.0.1");
    cmd.arg("--port");
    cmd.arg(port.to_string());
    cmd.env("AITEXT_PROD_DATA_DIR", data_dir);
    if !bundle_root.is_empty() {
        cmd.env("PYTHONPATH", bundle_root);
    }
    cmd.env("PYTHONIOENCODING", "utf-8");
    cmd.env("PYTHONUNBUFFERED", "1");
    cmd.env("HF_HUB_OFFLINE", "1");
    cmd.env("TRANSFORMERS_OFFLINE", "1");

    let _ = cmd.spawn();
    println!("[PlotPilot] Backend started with Python: {}", python_exe);
}

#[cfg(target_os = "linux")]
fn launch_linux_backend_impl(bundle_root: &str, data_dir: &str, port: u16) {
    let mut cmd = Command::new("python3");
    cmd.arg("-m");
    cmd.arg("uvicorn");
    cmd.arg("interfaces.main:app");
    cmd.arg("--host");
    cmd.arg("127.0.0.1");
    cmd.arg("--port");
    cmd.arg(port.to_string());
    cmd.env("AITEXT_PROD_DATA_DIR", data_dir);
    cmd.env("PYTHONIOENCODING", "utf-8");
    cmd.env("HF_HUB_OFFLINE", "1");
    cmd.env("TRANSFORMERS_OFFLINE", "1");
    if !bundle_root.is_empty() {
        cmd.env("PYTHONPATH", bundle_root);
    }
    let _ = cmd.spawn();
}

// ─────────────────────────────────────────────────────────────────────────────

/// 解析平台数据目录
#[cfg(target_os = "macos")]
fn resolve_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|p| p.join("PlotPilot"))
        .unwrap_or_else(|| PathBuf::from("/tmp/PlotPilot"))
}

#[cfg(target_os = "windows")]
fn resolve_data_dir() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|p| p.join("PlotPilot"))
        .unwrap_or_else(|_| PathBuf::from("C:\\temp\\PlotPilot"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn resolve_data_dir() -> PathBuf {
    std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .map(|p| p.join("plotpilot"))
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|p| p.join(".local/share/plotpilot"))
                .unwrap_or_else(|| PathBuf::from("/tmp/plotpilot"))
        })
}

/// 查找空闲端口
fn find_free_port(start: u16, max_try: u16) -> u16 {
    use std::net::TcpListener;

    for port in start..start.saturating_add(max_try) {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    eprintln!("[PlotPilot] Warning: No free port in range {}-{}, falling back to {}", start, start.saturating_add(max_try), start);
    start
}
