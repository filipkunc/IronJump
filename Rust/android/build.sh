#!/usr/bin/env bash
# Build a debug-signed release APK using a pinned, locally patched packaging tool.
set -euo pipefail
rust_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export NDK_HOME="${NDK_HOME:-$ANDROID_HOME/ndk/28.2.13676358}"
if [[ ! -f "$ANDROID_HOME/platforms/android-36/android.jar" || ! -d "$NDK_HOME" ]]; then
    echo 'Install SDK platform 36, build-tools 36, and NDK 28.2.13676358, or set ANDROID_HOME and NDK_HOME.' >&2
    exit 1
fi
revision=d411c8fe1c08339e46d7dc6b40ac04ea5e3a01b6
patch_file="$rust_dir/android/cargo-quad-apk-rust2024.patch"
patch_hash="$(sha256sum "$patch_file" | cut -d ' ' -f 1)"
tool_dir="$rust_dir/target/android-tools/$revision-$patch_hash"
tool="$tool_dir/cargo-quad-apk"
if [[ ! -x "$tool" ]]; then
    mkdir -p "$tool_dir"
    if [[ ! -d "$tool_dir/source/.git" ]]; then
        git init -q "$tool_dir/source"
        git -C "$tool_dir/source" remote add origin https://github.com/not-fl3/cargo-quad-apk.git
    fi
    git -C "$tool_dir/source" fetch --depth 1 origin "$revision"
    git -C "$tool_dir/source" checkout --detach "$revision"
    if git -C "$tool_dir/source" apply --check "$patch_file"; then
        git -C "$tool_dir/source" apply "$patch_file"
    else
        git -C "$tool_dir/source" apply --reverse --check "$patch_file"
    fi
    cargo build --release --locked --manifest-path "$tool_dir/source/Cargo.toml"
    cp "$tool_dir/source/target/release/cargo-quad-apk" "$tool"
fi
cd "$rust_dir"
"$tool" quad-apk build --release --locked --manifest-path "$rust_dir/Cargo.toml" "$@"
echo "APK: $rust_dir/target/android-artifacts/release/apk/ironjump.apk"
