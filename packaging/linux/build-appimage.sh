#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARCH="${ARCH:-$(uname -m)}"
APP_NAME="OpenHID-BT"
APPDIR="${ROOT_DIR}/target/appimage/${APP_NAME}.AppDir"
DIST_DIR="${ROOT_DIR}/dist"
VERSION="${VERSION:-$(sed -n 's/^version = "\([^"]*\)"/\1/p' "${ROOT_DIR}/Cargo.toml" | head -n1)}"
OUTPUT="${DIST_DIR}/${APP_NAME}-${VERSION}-linux-${ARCH}.AppImage"

echo "Building ${APP_NAME} v${VERSION} (${ARCH}) AppImage..."

cd "${ROOT_DIR}"
cargo build --release \
  -p openhid \
  -p openhid-desktop \
  -p openhid-agent \
  -p openhid-overlay

rm -rf "${APPDIR}"
mkdir -p \
  "${APPDIR}/usr/bin" \
  "${APPDIR}/usr/share/applications" \
  "${APPDIR}/usr/share/openhid/udev" \
  "${APPDIR}/usr/share/icons/hicolor/128x128/apps" \
  "${APPDIR}/usr/share/icons/hicolor/256x256/apps" \
  "${APPDIR}/usr/share/icons/hicolor/512x512/apps" \
  "${DIST_DIR}"

install -m 0755 "${ROOT_DIR}/target/release/openhid-desktop" "${APPDIR}/usr/bin/openhid-desktop"
install -m 0755 "${ROOT_DIR}/target/release/openhid-agent" "${APPDIR}/usr/bin/openhid-agent"
install -m 0755 "${ROOT_DIR}/target/release/openhid-overlay" "${APPDIR}/usr/bin/openhid-overlay"
install -m 0755 "${ROOT_DIR}/target/release/openhid" "${APPDIR}/usr/bin/openhid"

install -m 0644 "${ROOT_DIR}/packaging/linux/desktop/openlogi.desktop" "${APPDIR}/openlogi.desktop"
install -m 0644 "${ROOT_DIR}/packaging/linux/desktop/openlogi.desktop" "${APPDIR}/usr/share/applications/openlogi.desktop"
install -m 0644 "${ROOT_DIR}/packaging/linux/udev/70-openlogi.rules" "${APPDIR}/usr/share/openhid/udev/70-openlogi.rules"

install -m 0644 "${ROOT_DIR}/design/icon/openlogi-128.png" "${APPDIR}/usr/share/icons/hicolor/128x128/apps/openlogi.png"
install -m 0644 "${ROOT_DIR}/design/icon/openlogi-256.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/openlogi.png"
install -m 0644 "${ROOT_DIR}/design/icon/openlogi-512.png" "${APPDIR}/usr/share/icons/hicolor/512x512/apps/openlogi.png"
install -m 0644 "${ROOT_DIR}/design/icon/openlogi-256.png" "${APPDIR}/openlogi.png"

cat > "${APPDIR}/AppRun" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="${HERE}/usr/bin:${PATH}"

if [[ "${1:-}" == "--udev-rules" ]]; then
  cat "${HERE}/usr/share/openhid/udev/70-openlogi.rules"
  exit 0
fi

if [[ "${1:-}" == "--cli" ]]; then
  shift
  exec "${HERE}/usr/bin/openhid" "$@"
fi

exec "${HERE}/usr/bin/openhid-desktop" "$@"
EOF
chmod 0755 "${APPDIR}/AppRun"

APPIMAGETOOL="${APPIMAGETOOL:-$(command -v appimagetool || true)}"
if [[ -z "${APPIMAGETOOL}" ]]; then
  echo "error: appimagetool not found on PATH (set APPIMAGETOOL=/path/to/appimagetool)" >&2
  exit 1
fi

ARCH="${ARCH}" "${APPIMAGETOOL}" --no-appstream "${APPDIR}" "${OUTPUT}"
sha256sum "${OUTPUT}" > "${OUTPUT}.sha256"

echo "Created AppImage: ${OUTPUT}"
