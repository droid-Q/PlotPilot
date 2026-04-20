#!/bin/bash
# PlotPilot macOS 后端启动脚本
# 由 Rust Tauri 应用调用
# Bundle 结构:
#   PlotPilot.app/
#   ├── Contents/
#   │   ├── MacOS/plotpilot
#   │   └── Resources/
#   │       ├── application/
#   │       ├── interfaces/
#   │       ├── infrastructure/
#   │       ├── domain/
#   │       ├── python_macos/
#   │       │   ├── aarch64/    ← Apple Silicon
#   │       │   └── x86_64/     ← Intel
#   │       └── scripts/start_mac.sh

set -e

PORT="${1:-8005}"
DATA_DIR="${2:-}"
BUNDLE_ROOT="${3:-}"

# 从 Rust 注入的环境变量获取 resource dir
RESOURCE_DIR="${PLOTPILOT_RESOURCE_DIR:-}"

# 如果没有传入，尝试从 bundle 结构推导
if [ -z "$RESOURCE_DIR" ] && [ -n "$BUNDLE_ROOT" ]; then
    RESOURCE_DIR="$BUNDLE_ROOT/Contents/Resources"
fi

# ── 检测 CPU 架构 ────────────────────────────────────────────────────────
detect_arch() {
    local arch
    arch=$(uname -m)
    if [ "$arch" = "arm64" ]; then
        echo "aarch64"
    else
        echo "x86_64"
    fi
}

PYTHON_ARCH=$(detect_arch)

# ── 定位 Python ───────────────────────────────────────────────────────────
find_python() {
    local resource_dir="$1"

    # 1) 优先: bundle 内的 embedded Python
    if [ -n "$resource_dir" ] && [ -d "$resource_dir/python_macos/$PYTHON_ARCH/bin" ]; then
        EMBEDDED_PY="$resource_dir/python_macos/$PYTHON_ARCH/bin/python3"
        if [ -x "$EMBEDDED_PY" ]; then
            # 验证 uvicorn 可用
            if "$EMBEDDED_PY" -c "import uvicorn" 2>/dev/null; then
                echo "$EMBEDDED_PY"
                return 0
            fi
        fi
    fi

    # 2) 系统 python3
    for cmd in python3 python; do
        PY_PATH=$(command -v "$cmd" 2>/dev/null || true)
        if [ -n "$PY_PATH" ] && "$PY_PATH" -c "import uvicorn" 2>/dev/null; then
            echo "$PY_PATH"
            return 0
        fi
    done

    return 1
}

# ── 主流程 ────────────────────────────────────────────────────────────────
echo "PlotPilot 后端启动器 (macOS)"
echo "[INFO] Bundle 根: ${BUNDLE_ROOT:-未设置}"
echo "[INFO] 资源目录: ${RESOURCE_DIR:-未设置}"
echo "[INFO] CPU 架构: $PYTHON_ARCH"
echo "[INFO] 数据目录: ${DATA_DIR:-未设置}"
echo "[INFO] 端口: $PORT"

PYTHON_EXE=$(find_python "$RESOURCE_DIR") || {
    echo "[ERROR] 未找到带 uvicorn 的 Python"
    echo "[ERROR] 请确保已安装依赖: pip install -r requirements.txt"
    exit 1
}
echo "[OK] 使用 Python: $PYTHON_EXE"

# ── 环境变量准备 ───────────────────────────────────────────────────────
export PYTHONIOENCODING="utf-8"
export PYTHONUNBUFFERED="1"
export HF_HUB_OFFLINE="1"
export TRANSFORMERS_OFFLINE="1"

if [ -n "$DATA_DIR" ]; then
    export AITEXT_PROD_DATA_DIR="$DATA_DIR"
fi

# PYTHONPATH: bundle 的 Resources 目录
if [ -n "$RESOURCE_DIR" ]; then
    export PYTHONPATH="$RESOURCE_DIR"
fi

echo "[OK] PYTHONPATH: ${PYTHONPATH:-未设置}"

# 确保数据目录存在
if [ -n "$DATA_DIR" ]; then
    mkdir -p "$DATA_DIR"
fi

# ── 启动 uvicorn ───────────────────────────────────────────────────────
BACKEND_LOG="${DATA_DIR:-/tmp}/plotpilot_backend.log"
echo "[OK] 后端日志: $BACKEND_LOG"

cd "$BUNDLE_ROOT" 2>/dev/null || cd /tmp

"$PYTHON_EXE" -m uvicorn \
    "interfaces.main:app" \
    --host 127.0.0.1 \
    --port "$PORT" \
    --log-level info \
    > "$BACKEND_LOG" 2>&1 &

BACKEND_PID=$!
echo "$BACKEND_PID" > /tmp/plotpilot_backend.pid
echo "[OK] 后端进程已启动 (PID=$BACKEND_PID)"

# ── 健康检查 ────────────────────────────────────────────────────────────
echo "[INFO] 等待服务就绪..."
for i in $(seq 1 60); do
    if curl -sf "http://127.0.0.1:$PORT/health" > /dev/null 2>&1; then
        echo "[OK] 服务已就绪 -> http://127.0.0.1:$PORT"
        exit 0
    fi
    sleep 1
done

echo "[ERROR] 服务启动超时，请查看: $BACKEND_LOG"
exit 1
