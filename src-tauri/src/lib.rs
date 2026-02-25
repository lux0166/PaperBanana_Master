use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonProcess {
    pub pid: Option<u32>,
    pub port: u16,
    pub running: bool,
}

pub struct AppState {
    pub python_process: Mutex<PythonProcess>,
}

#[tauri::command]
async fn get_backend_status(state: State<'_, AppState>) -> Result<PythonProcess, String> {
    let process = state
        .python_process
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(process.clone())
}

#[tauri::command]
async fn start_python_backend(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tauri_plugin_shell::ShellExt;

    let mut process = state
        .python_process
        .lock()
        .map_err(|e| e.to_string())?;

    if process.running {
        return Ok(format!("Backend already running on port {}", process.port));
    }

    let port = process.port;

    // Get the resource directory to find bundled Python files
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?;

    let demo_py = resource_dir.join("demo.py");
    let demo_path = if demo_py.exists() {
        demo_py.to_string_lossy().to_string()
    } else {
        // Fallback: try relative path (dev mode)
        "demo.py".to_string()
    };

    let shell = app.shell();
    let output = shell
        .command("streamlit")
        .args([
            "run",
            &demo_path,
            "--server.port",
            &port.to_string(),
            "--server.address",
            "127.0.0.1",
            "--server.headless",
            "true",
        ])
        .spawn();

    match output {
        Ok((_rx, child)) => {
            process.pid = Some(child.pid());
            process.running = true;
            Ok(format!("Backend started on port {}", port))
        }
        Err(e) => Err(format!("Failed to start backend: {}", e)),
    }
}

#[tauri::command]
async fn stop_python_backend(state: State<'_, AppState>) -> Result<String, String> {
    let mut process = state
        .python_process
        .lock()
        .map_err(|e| e.to_string())?;

    if !process.running {
        return Ok("Backend is not running".to_string());
    }

    // On Windows, kill the process tree
    if let Some(pid) = process.pid {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();
        }
    }

    process.running = false;
    process.pid = None;
    Ok("Backend stopped".to_string())
}

#[tauri::command]
async fn check_python_installed() -> Result<bool, String> {
    let output = std::process::Command::new("python")
        .args(["--version"])
        .output();

    match output {
        Ok(o) => Ok(o.status.success()),
        Err(_) => {
            // Try python3 as fallback
            let output = std::process::Command::new("python3")
                .args(["--version"])
                .output();
            match output {
                Ok(o) => Ok(o.status.success()),
                Err(_) => Ok(false),
            }
        }
    }
}

#[tauri::command]
async fn install_dependencies() -> Result<String, String> {
    let output = std::process::Command::new("pip")
        .args(["install", "-r", "requirements.txt"])
        .output()
        .map_err(|e| format!("Failed to run pip: {}", e))?;

    if output.status.success() {
        Ok("Dependencies installed successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to install dependencies: {}", stderr))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            python_process: Mutex::new(PythonProcess {
                pid: None,
                port: 8501,
                running: false,
            }),
        })
        .invoke_handler(tauri::generate_handler![
            get_backend_status,
            start_python_backend,
            stop_python_backend,
            check_python_installed,
            install_dependencies,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Stop the Python backend when the window is closed
                let app_state = window.state::<AppState>();
                if let Ok(mut process) = app_state.python_process.lock() {
                    if let Some(pid) = process.pid {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = std::process::Command::new("taskkill")
                                .args(["/F", "/T", "/PID", &pid.to_string()])
                                .output();
                        }
                        #[cfg(not(target_os = "windows"))]
                        {
                            let _ = std::process::Command::new("kill")
                                .args(["-9", &pid.to_string()])
                                .output();
                        }
                        process.running = false;
                        process.pid = None;
                    }
                };
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
