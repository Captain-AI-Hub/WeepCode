#!/bin/sh

set -eu

repository_url="https://github.com/Captain-AI-Hub/WeepCode"

operating_system="$(uname -s)"
machine_architecture="$(uname -m)"

case "$operating_system" in
    Linux)
        case "$machine_architecture" in
            x86_64 | amd64) platform="linux-x86_64" ;;
            aarch64 | arm64) platform="linux-aarch64" ;;
            *)
                echo "WeepCode does not provide a Linux package for architecture: $machine_architecture" >&2
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$machine_architecture" in
            aarch64 | arm64) platform="macos-aarch64" ;;
            *)
                echo "WeepCode currently supports macOS only on Apple Silicon." >&2
                exit 1
                ;;
        esac
        ;;
    *)
        echo "WeepCode does not provide a Unix package for: $operating_system" >&2
        exit 1
        ;;
esac

asset_name="weepcode-$platform.tar.gz"
if [ -n "${WEEPCODE_VERSION:-}" ]; then
    download_base_url="$repository_url/releases/download/$WEEPCODE_VERSION"
else
    download_base_url="$repository_url/releases/latest/download"
fi

if [ -n "${WEEPCODE_INSTALL_DIR:-}" ]; then
    install_directory="$WEEPCODE_INSTALL_DIR"
elif [ -n "${HOME:-}" ]; then
    install_directory="$HOME/.local/bin"
else
    echo "HOME is not set; set WEEPCODE_INSTALL_DIR to choose an installation directory." >&2
    exit 1
fi

temporary_directory="$(mktemp -d 2>/dev/null || mktemp -d -t weepcode-install)"
trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

archive_path="$temporary_directory/$asset_name"
checksums_path="$temporary_directory/SHA256SUMS"

curl -fL --retry 3 --proto '=https' --tlsv1.2 \
    "$download_base_url/$asset_name" -o "$archive_path"
curl -fL --retry 3 --proto '=https' --tlsv1.2 \
    "$download_base_url/SHA256SUMS" -o "$checksums_path"

expected_checksum="$(awk -v asset="$asset_name" '$2 == asset { print $1 }' "$checksums_path")"
if [ -z "$expected_checksum" ]; then
    echo "No checksum was published for $asset_name." >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    actual_checksum="$(sha256sum "$archive_path" | awk '{ print $1 }')"
elif command -v shasum >/dev/null 2>&1; then
    actual_checksum="$(shasum -a 256 "$archive_path" | awk '{ print $1 }')"
else
    echo "A SHA-256 utility (sha256sum or shasum) is required." >&2
    exit 1
fi

if [ "$actual_checksum" != "$expected_checksum" ]; then
    echo "Checksum verification failed for $asset_name." >&2
    exit 1
fi

extraction_directory="$temporary_directory/extracted"
mkdir -p "$extraction_directory" "$install_directory"
tar -xzf "$archive_path" -C "$extraction_directory"
install -m 0755 "$extraction_directory/weepcode" "$install_directory/weepcode"

echo "WeepCode installed to $install_directory/weepcode"
case ":${PATH:-}:" in
    *":$install_directory:"*) ;;
    *)
        echo "Add $install_directory to PATH, then restart your shell:"
        echo "  export PATH=\"$install_directory:\$PATH\""
        ;;
esac
