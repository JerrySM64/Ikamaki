#   make app        native GUI binary   -> target/release/ikamaki
#   make web        WASM web bundle     -> dist/web/ (serve this directory)
#   make clean      remove target/ and dist/
#   make clean-all  also drop the podman cache volumes

PODMAN       ?= podman
FEDORA_IMAGE ?= docker.io/library/fedora:latest

CARGO_VOLUME  = ikamaki-cargo
RUSTUP_VOLUME = ikamaki-rustup
DNF_VOLUME    = ikamaki-dnf

PODMAN_RUN = $(PODMAN) run --rm \
	--volume "$(CURDIR)":/src:Z \
	--volume $(CARGO_VOLUME):/root/.cargo \
	--volume $(RUSTUP_VOLUME):/root/.rustup \
	--volume $(DNF_VOLUME):/var/cache/dnf \
	--workdir /src \
	$(FEDORA_IMAGE)

define container_setup
set -euo pipefail
dnf install -y clang gcc gcc-c++ make pkgconf-pkg-config tar gzip curl-minimal openssl-devel dbus-devel fontconfig-devel freetype-devel libxcb-devel libxkbcommon-devel libxkbcommon-x11-devel mesa-libGL-devel mesa-libEGL-devel wayland-devel
if [ ! -x /root/.cargo/bin/rustup ]; then
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
fi
. /root/.cargo/env
rustc --version
cargo --version
endef

define app_script
$(container_setup)
CARGO_TARGET_DIR=/src/target cargo build --release
endef
export app_script

define web_script
$(container_setup)
rustup target add wasm32-unknown-unknown
CARGO_TARGET_DIR=/src/target cargo build --release --target wasm32-unknown-unknown
wasm_bindgen_version=$$(cargo tree --target wasm32-unknown-unknown --invert wasm-bindgen --prefix none | sed -n 's/^wasm-bindgen v//p' | head -n 1)
echo "wasm-bindgen: $$wasm_bindgen_version"
if ! wasm-bindgen --version 2>/dev/null | grep -q "$$wasm_bindgen_version"; then
	CARGO_TARGET_DIR=/tmp/wasm-bindgen-cli cargo install wasm-bindgen-cli --version "$$wasm_bindgen_version" --locked --force
fi
mkdir -p /src/dist/web
wasm-bindgen --target web --no-typescript --out-dir /src/dist/web --out-name ikamaki /src/target/wasm32-unknown-unknown/release/ikamaki.wasm
cat > /src/dist/web/index.html <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=no">
<title>Ikamaki</title>
<style>
html, body { overflow: hidden; margin: 0; padding: 0; height: 100%; width: 100%; background: rgb(64, 64, 64); }
canvas { display: block; position: absolute; top: 0; left: 0; width: 100% !important; height: 100% !important; }
.loading_text { position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); color: rgb(240, 240, 240); font: 16px sans-serif; }
</style>
</head>
<body>
<div class="loading_text">Loading Ikamaki...</div>
<canvas id="canvas"></canvas>
<script type="module">
import init from "./ikamaki.js";
init();
</script>
</body>
</html>
HTML
endef
export web_script

.DEFAULT_GOAL := app

.PHONY: app web clean clean-all

app:
	$(PODMAN_RUN) bash -c "$$app_script"
	@echo "Built native binary: target/release/ikamaki"

web:
	$(PODMAN_RUN) bash -c "$$web_script"
	@echo "Built web bundle: dist/web"
	@echo "Serve it with: python3 -m http.server -d dist/web"

clean:
	rm -rf target dist

clean-all: clean
	-$(PODMAN) volume rm -f $(CARGO_VOLUME) $(RUSTUP_VOLUME) $(DNF_VOLUME)
