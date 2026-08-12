#!/usr/bin/env bash

set -euo pipefail

readonly MDBOOK_VERSION='0.5.4'
readonly MDBOOK_SHA256='3f28de05dafca9d0f2eab99c662116b0e37b89b1d96a08f8f430b9eeae958cd7'
readonly LYCHEE_VERSION='0.24.2'
readonly LYCHEE_SHA256='73657a111819a30c47c08352896796f23d64e4eb2b3ed39b6d32149241566fc5'
readonly PUBLIC_API_VERSION='0.52.0'
readonly PUBLIC_API_NIGHTLY='nightly-2026-03-12'
readonly TOOLS_ROOT="${VMNL_API_TOOLS_DIR:-$PWD/target/api-tools}"
readonly BIN_DIR="$TOOLS_ROOT/bin"
readonly DOWNLOAD_DIR="$TOOLS_ROOT/downloads"
TEMP_ARCHIVE=''
TEMP_UNPACK_DIR=''

cleanup_temporary_files()
{
    if [[ -n $TEMP_ARCHIVE ]]; then
        rm -f -- "$TEMP_ARCHIVE"
        TEMP_ARCHIVE=''
    fi
    if [[ -n $TEMP_UNPACK_DIR ]]; then
        rm -rf -- "$TEMP_UNPACK_DIR"
        TEMP_UNPACK_DIR=''
    fi
}

trap cleanup_temporary_files EXIT

die()
{
    printf 'API documentation tools: %s\n' "$*" >&2
    exit 1
}

require_command()
{
    command -v "$1" >/dev/null 2>&1 || die "missing prerequisite '$1'"
}

has_version()
{
    local command_name=$1
    local expected=$2

    command -v "$command_name" >/dev/null 2>&1 \
        && "$command_name" --version 2>/dev/null | grep -Fq "$expected"
}

install_archive_tool()
{
    local tool_name=$1
    local version=$2
    local url=$3
    local checksum=$4
    local binary

    if has_version "$tool_name" "$version"; then
        return
    fi

    TEMP_ARCHIVE=$(mktemp "$DOWNLOAD_DIR/$tool_name.XXXXXX.tar.gz")
    TEMP_UNPACK_DIR=$(mktemp -d "$DOWNLOAD_DIR/$tool_name.XXXXXX")
    curl --fail --location --silent --show-error --output "$TEMP_ARCHIVE" "$url"
    printf '%s  %s\n' "$checksum" "$TEMP_ARCHIVE" | sha256sum --check
    tar -xzf "$TEMP_ARCHIVE" -C "$TEMP_UNPACK_DIR"
    binary=$(find "$TEMP_UNPACK_DIR" -type f -name "$tool_name" -print -quit)
    [[ -n $binary ]] || die "archive for $tool_name $version contains no '$tool_name' binary"
    install -m 0755 "$binary" "$BIN_DIR/$tool_name"
    cleanup_temporary_files
}

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) ;;
    *) die 'automatic installation currently supports Linux x86_64 only; install the pinned tools from docs/build.md' ;;
esac

for command_name in curl find grep install mktemp sha256sum tar uname; do
    require_command "$command_name"
done

rustc "+$PUBLIC_API_NIGHTLY" --version >/dev/null 2>&1 \
    || die "missing $PUBLIC_API_NIGHTLY; run 'rustup toolchain install $PUBLIC_API_NIGHTLY --profile minimal'"

mkdir -p "$BIN_DIR" "$DOWNLOAD_DIR"
export PATH="$BIN_DIR:$PATH"

install_archive_tool \
    mdbook \
    "$MDBOOK_VERSION" \
    "https://github.com/rust-lang/mdBook/releases/download/v$MDBOOK_VERSION/mdbook-v$MDBOOK_VERSION-x86_64-unknown-linux-gnu.tar.gz" \
    "$MDBOOK_SHA256"

install_archive_tool \
    lychee \
    "$LYCHEE_VERSION" \
    "https://github.com/lycheeverse/lychee/releases/download/lychee-v$LYCHEE_VERSION/lychee-x86_64-unknown-linux-musl.tar.gz" \
    "$LYCHEE_SHA256"

if ! has_version cargo-public-api "$PUBLIC_API_VERSION"; then
    cargo "+$PUBLIC_API_NIGHTLY" install \
        cargo-public-api \
        --version "$PUBLIC_API_VERSION" \
        --locked \
        --root "$TOOLS_ROOT"
fi

printf 'API documentation tools ready in %s\n' "$BIN_DIR"
