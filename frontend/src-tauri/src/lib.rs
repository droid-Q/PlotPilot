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

#[cfg(target_os = "macos")]
fn launch_mac_backend_impl(bundle_root: &str, data_dir: &str, port: u16) {
    // macOS bundle 结构: PlotPilot.app/Contents/Resources/
    // Python 代码和脚本都在 Resources/ 下
    let resource_dir = if !bundle_root.is_empty() {
        PathBuf::from(bundle_root).join("Contents/Resources")
    } else {
        // Dev mode: fallback to current working directory
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };
    let script = resource_dir.join("scripts/start_mac.sh");

    if script.exists() {
        // 通过环境变量传递 resource_dir（避免增加命令行参数）
        let mut cmd = Command::new("/bin/bash");
        cmd.arg(&script);
        cmd.arg(port.to_string());
        cmd.arg(data_dir);
        cmd.arg(bundle_root);
        cmd.env("PLOTPILOT_RESOURCE_DIR", &resource_dir);
        cmd.env("AITEXT_PROD_DATA_DIR", data_dir);
        cmd.env("PYTHONPATH", &resource_dir);
        cmd.env("PYTHONIOENCODING", "utf-8");
        cmd.env("PYTHONUNBUFFERED", "1");
        cmd.env("HF_HUB_OFFLINE", "1");
        cmd.env("TRANSFORMERS_OFFLINE", "1");
        let _ = cmd.spawn();
        println!("[PlotPilot] Backend launch script: {:?}", script);
    } else {
        eprintln!(
            "[PlotPilot] Backend script not found: {:?}, falling back to python3",
            script
        );
        // Fallback: 直接调用 python3，PYTHONPATH 指向 Resources
        let mut cmd = Command::new("python3");
        cmd.arg("-m");
        cmd.arg("uvicorn");
        cmd.arg("interfaces.main:app");
        cmd.arg("--host");
        cmd.arg("127.0.0.1");
        cmd.arg("--port");
        cmd.arg(port.to_string());
        cmd.env("AITEXT_PROD_DATA_DIR", data_dir);
        cmd.env("PYTHONPATH", &resource_dir);
        cmd.env("PYTHONIOENCODING", "utf-8");
        cmd.env("PYTHONUNBUFFERED", "1");
        cmd.env("HF_HUB_OFFLINE", "1");
        cmd.env("TRANSFORMERS_OFFLINE", "1");
        let _ = cmd.spawn();
    }
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
