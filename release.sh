#!/bin/bash
set -e

# hwmon v0.1.1 GitHub Release Script
# Usage: GITHUB_TOKEN=xxx bash release.sh

if [ -z "$GITHUB_TOKEN" ]; then
    echo "ERROR: GITHUB_TOKEN not set"
    echo "Usage: GITHUB_TOKEN=your_token bash release.sh"
    exit 1
fi

REPO="autumnQWQ/hwmon"
TAG="v0.1.1"
VERSION="0.1.1"
ZIP="hwmon-v${VERSION}-win64.zip"
ZIP_PATH="$(cd "$(dirname "$0")" && pwd)/${ZIP}"

echo "=== hwmon v${VERSION} Release Script ==="
echo "Repo:    ${REPO}"
echo "Tag:     ${TAG}"
echo "Zip:     ${ZIP_PATH}"
echo ""

# Check zip exists
if [ ! -f "$ZIP_PATH" ]; then
    echo "ERROR: ${ZIP} not found in script directory"
    echo "Run package.bat first to create it"
    exit 1
fi
echo "[OK] Zip found: $(du -h "$ZIP_PATH" | cut -f1)"

# Check uncommitted changes
echo ""
echo "WARNING: Make sure you've committed and pushed all changes first!"
echo "  git add -A && git commit -m \"...\" && git push origin main"
echo ""
read -p "Continue? (y/N): " CONFIRM
if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "[1/3] Creating GitHub Release..."

RELEASE_RESP=$(curl -s -w "\n%{http_code}" -X POST "https://api.github.com/repos/${REPO}/releases" \
    -H "Authorization: token ${GITHUB_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
        "tag_name": "'"${TAG}"'",
        "target_commitish": "main",
        "name": "v'${VERSION}' - Self-contained Windows hardware monitor",
        "body": "## v0.1.1 - 自包含打包发布\n\n### 🎉 新特性\n- **自包含打包** — `dist-win/hwmon/` 解压即用，无需 Rust / Node.js / 任何外部依赖\n- **package.bat** — 自动化打包脚本，一键生成可发布的 zip\n- **Electron 路径优化** — 支持打包分发 + 开发环境双模式\n\n### 🛠 修复\n- Electron 启动路径现在相对于 `hwmon.exe` 所在目录，支持任意位置运行\n- Electron 查找策略：`hwmon-electron/electron/` → `node_modules/electron/dist/`\n\n### 系统要求\n- Windows 10/11 (x64)\n- 无需管理员权限\n- 无任何外部依赖\n\n### 📦 下载\n- `hwmon-v0.1.1-win64.zip` — 完整包 (含 Electron GUI 运行时)\n- 解压后双击 `hwmon.exe` 即可启动悬浮窗",
        "draft": false,
        "prerelease": false
    }')

HTTP_CODE=$(echo "$RELEASE_RESP" | tail -n1)
BODY=$(echo "$RELEASE_RESP" | sed '$d')

if [ "$HTTP_CODE" != "201" ]; then
    echo "ERROR: Failed to create release (HTTP ${HTTP_CODE})"
    echo "$BODY"
    exit 1
fi

RELEASE_ID=$(echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null || \
             python -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null || \
             grep -o '"id":[0-9]*' <<< "$BODY" | head -1 | cut -d: -f2)
echo "[OK] Release created: ID ${RELEASE_ID}"

echo ""
echo "[2/3] Uploading zip asset..."
ASSET_RESP=$(curl -s -w "\n%{http_code}" -X POST \
    "https://uploads.github.com/repos/${REPO}/releases/${RELEASE_ID}/assets?name=${ZIP}" \
    -H "Authorization: token ${GITHUB_TOKEN}" \
    -H "Content-Type: application/octet-stream" \
    --data-binary "@${ZIP_PATH}")

HTTP_CODE=$(echo "$ASSET_RESP" | tail -n1)
BODY=$(echo "$ASSET_RESP" | sed '$d')

if [ "$HTTP_CODE" != "201" ]; then
    echo "ERROR: Failed to upload asset (HTTP ${HTTP_CODE})"
    echo "$BODY"
    exit 1
fi

echo "[OK] Zip uploaded successfully!"

echo ""
echo "[3/3] Pushing tag..."
git tag -f "$TAG" 2>/dev/null
git push origin "$TAG" -f 2>&1 || true

echo ""
echo "=========================================="
echo "  Release v${VERSION} complete!"
echo "=========================================="
echo "  https://github.com/${REPO}/releases/tag/${TAG}"
echo ""
