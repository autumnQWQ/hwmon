# hwmon — 极简硬件监控
# Makefile for build + packaging (macOS, Linux, Windows via cross)

.PHONY: build release clean package package-mac package-win install uninstall help test

# ─── Variables ────────────────────────────────────────────
BINARY   := hwmon
VERSION  := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
RELEASE  := target/release/$(BINARY)
OS       := $(shell uname -s | tr '[:upper:]' '[:lower:]')
ARCH     := $(shell uname -m)
# Normalize arch names
ifeq ($(ARCH),x86_64)
    ARCH := x64
endif
ifeq ($(ARCH),aarch64)
    ARCH := arm64
endif
TARBALL  := $(BINARY)-v$(VERSION)-$(OS)-$(ARCH).tar.gz
ZIPFILE  := $(BINARY)-v$(VERSION)-win64.zip
INSTALL_DIR ?= /usr/local/bin

# ─── Build ────────────────────────────────────────────────

help:
	@echo "hwmon v$(VERSION) — 极简硬件监控"
	@echo ""
	@echo "  make build         Debug build"
	@echo "  make release       Optimized release build"
	@echo "  make test          Run quick smoke test"
	@echo "  make clean         Remove build artifacts"
	@echo "  make package       Package for current platform"
	@echo "  make package-all   Package for all targets"
	@echo "  make install       Install to $(INSTALL_DIR)"
	@echo "  make uninstall     Remove from $(INSTALL_DIR)"
	@echo ""

build:
	cargo build

release:
	cargo build --release
	@echo ""
	@ls -lh $(RELEASE)

test: release
	@echo "=== JSON mode ==="
	@$(RELEASE) --json 2>&1 | head -5
	@echo ""
	@echo "=== Terminal mode ==="
	@$(RELEASE) 2>&1

clean:
	cargo clean
	rm -rf dist/

# ─── Packaging ────────────────────────────────────────────

dist:
	mkdir -p dist

package: release dist
	@echo "📦 Packaging $(TARBALL)..."
	cp $(RELEASE) dist/$(BINARY)
	cp install.sh dist/
	cp install.bat dist/ 2>/dev/null || true
	cd dist && tar czf ../$(TARBALL) $(BINARY) install.sh install.bat 2>/dev/null
	@ls -lh $(TARBALL)
	@echo "✅ $(TARBALL) ready"

package-mac: release dist
	@echo "📦 Creating macOS .pkg..."
	cp $(RELEASE) dist/$(BINARY)
	mkdir -p dist/pkg-root/usr/local/bin
	cp $(RELEASE) dist/pkg-root/usr/local/bin/$(BINARY)
	pkgbuild --root dist/pkg-root \
		--identifier com.hwmon.cli \
		--version $(VERSION) \
		--install-location / \
		dist/$(BINARY)-v$(VERSION).pkg 2>/dev/null || \
		(echo "⚠️  pkgbuild failed — using tar.gz instead" && \
		 cd dist && tar czf ../$(TARBALL) $(BINARY) install.sh)
	@ls -lh dist/$(BINARY)-v$(VERSION).pkg 2>/dev/null || ls -lh $(TARBALL)
	@echo "✅ macOS package ready"

package-win:
	@echo "📦 Creating Windows zip..."
	cargo build --release --target x86_64-pc-windows-msvc 2>&1 || \
		(echo "⚠️  Cross-compile not available — build on Windows directly" && exit 1)
	mkdir -p dist
	cp target/x86_64-pc-windows-msvc/release/hwmon.exe dist/
	cp install.bat dist/
	cd dist && zip -r ../$(ZIPFILE) hwmon.exe install.bat
	@ls -lh $(ZIPFILE)
	@echo "✅ $(ZIPFILE) ready"

package-all: package-mac
	@echo ""
	@echo "📋 Packaging summary:"
	@ls -lh $(TARBALL) dist/*.pkg 2>/dev/null || true
	@echo ""
	@echo "Windows: run 'make package-win' on a Windows machine"
	@echo "  or: cargo build --release --target x86_64-pc-windows-msvc"
	@echo "  or on Windows: build-release.bat"

# ─── Install / Uninstall ──────────────────────────────────

install: release
	@echo "🔧 Installing hwmon v$(VERSION) to $(INSTALL_DIR)..."
	install -d "$(INSTALL_DIR)"
	install -m 755 $(RELEASE) "$(INSTALL_DIR)/$(BINARY)"
	@echo "✅ Installed. Run: hwmon"

uninstall:
	@echo "🗑 Removing hwmon..."
	rm -f "$(INSTALL_DIR)/$(BINARY)"
	@echo "✅ Removed"
