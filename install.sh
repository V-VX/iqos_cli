#!/bin/sh
set -eu

OWNER="V-VX"
REPO="iqos_cli"
API_ROOT="${IQOS_CLI_GITHUB_API_ROOT:-https://api.github.com}"
VERSION="${IQOS_CLI_VERSION:-latest}"
INSTALL_DIR="${IQOS_CLI_INSTALL_DIR:-}"

info() {
    printf '%s\n' "info: $*"
}

warn() {
    printf '%s\n' "warning: $*" >&2
}

die() {
    printf '%s\n' "error: $*" >&2
    exit 1
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"
}

has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

http_get() {
    url=$1

    if has_cmd curl; then
        curl -fsSL -H 'Accept: application/vnd.github+json' "$url"
    elif has_cmd wget; then
        wget -qO- --header='Accept: application/vnd.github+json' "$url"
    else
        die "Neither curl nor wget is installed. Install one of them and try again."
    fi
}

http_download() {
    url=$1
    output=$2

    if has_cmd curl; then
        curl -fL --progress-bar "$url" -o "$output"
    elif has_cmd wget; then
        wget -O "$output" "$url"
    else
        die "Neither curl nor wget is installed. Install one of them and try again."
    fi
}

json_string_value() {
    field=$1
    sed -n "s/.*\"${field}\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*/\1/p" | head -n 1
}

asset_url_from_release() {
    release_json=$1
    asset_name=$2

    printf '%s' "$release_json" |
        tr ',' '\n' |
        awk -v name="$asset_name" '
            index($0, "\"name\":\"" name "\"") || index($0, "\"name\": \"" name "\"") {
                found = 1
            }
            found && match($0, /"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]+"/) {
                value = substr($0, RSTART, RLENGTH)
                sub(/^"browser_download_url"[[:space:]]*:[[:space:]]*"/, "", value)
                sub(/"$/, "", value)
                print value
                exit
            }
        '
}

normalize_version() {
    case "$VERSION" in
        "" | latest)
            printf '%s\n' "latest"
            ;;
        v*)
            printf '%s\n' "$VERSION"
            ;;
        *)
            printf '%s\n' "v$VERSION"
            ;;
    esac
}

release_api_url() {
    tag=$1

    if [ "$tag" = "latest" ]; then
        printf '%s\n' "${API_ROOT}/repos/${OWNER}/${REPO}/releases/latest"
    else
        printf '%s\n' "${API_ROOT}/repos/${OWNER}/${REPO}/releases/tags/${tag}"
    fi
}

normalize_arch() {
    case "$1" in
        x86_64 | amd64)
            printf '%s\n' "x86_64"
            ;;
        arm64 | aarch64)
            printf '%s\n' "aarch64"
            ;;
        *)
            die "Unsupported architecture: $1"
            ;;
    esac
}

detect_linux_libc() {
    if has_cmd getconf && getconf GNU_LIBC_VERSION >/dev/null 2>&1; then
        printf '%s\n' "gnu"
        return
    fi

    if has_cmd ldd; then
        ldd_output=$(ldd --version 2>&1 || true)
        case "$ldd_output" in
            *musl* | *Musl*)
                printf '%s\n' "musl"
                return
                ;;
            *GNU* | *glibc* | *GLIBC*)
                printf '%s\n' "gnu"
                return
                ;;
        esac
    fi

    if ls /lib/ld-musl-*.so.1 /usr/lib/ld-musl-*.so.1 >/dev/null 2>&1; then
        printf '%s\n' "musl"
        return
    fi

    die "Could not determine Linux libc. Set IQOS_CLI_TARGET to linux-<arch>-gnu or linux-<arch>-musl."
}

target_candidates() {
    if [ -n "${IQOS_CLI_TARGET:-}" ]; then
        printf '%s\n' "$IQOS_CLI_TARGET"
        return
    fi

    os=$(uname -s)
    arch=$(normalize_arch "$(uname -m)")

    case "$os" in
        Darwin)
            printf '%s\n' "macos-universal"
            ;;
        Linux)
            libc=$(detect_linux_libc)
            printf '%s\n' "linux-${arch}-${libc}"

            # v1.0.x releases used this package name for x86_64 glibc Linux.
            if [ "$arch" = "x86_64" ] && [ "$libc" = "gnu" ]; then
                printf '%s\n' "linux-x86_64"
            fi
            ;;
        *)
            die "Unsupported OS: $os"
            ;;
    esac
}

archive_name() {
    tag=$1
    package=$2

    case "$package" in
        macos-* | linux-*)
            printf '%s\n' "iqos_cli-${tag}-${package}.tar.gz"
            ;;
        *)
            die "Unsupported package target for install.sh: $package"
            ;;
    esac
}

select_asset() {
    release_json=$1
    tag=$2

    candidates=$(target_candidates)
    for package in $candidates; do
        asset=$(archive_name "$tag" "$package")
        url=$(asset_url_from_release "$release_json" "$asset")
        if [ -n "$url" ]; then
            printf '%s\n%s\n%s\n' "$package" "$asset" "$url"
            return
        fi
    done

    printf '%s\n' "$candidates" | sed 's/^/  - /' >&2
    die "No compatible release asset was found for this system."
}

sha256_file() {
    path=$1

    if has_cmd sha256sum; then
        sha256sum "$path" | awk '{print $1}'
    elif has_cmd shasum; then
        shasum -a 256 "$path" | awk '{print $1}'
    else
        return 1
    fi
}

verify_checksum() {
    release_json=$1
    asset_name=$2
    archive_path=$3
    tmp_dir=$4

    sums_url=$(asset_url_from_release "$release_json" "SHA256SUMS.txt")
    if [ -z "$sums_url" ]; then
        warn "SHA256SUMS.txt was not found in the release; skipping checksum verification."
        return
    fi

    if ! sha256_file "$archive_path" >/dev/null 2>&1; then
        warn "sha256sum or shasum was not found; skipping checksum verification."
        return
    fi

    sums_path="${tmp_dir}/SHA256SUMS.txt"
    if ! http_download "$sums_url" "$sums_path"; then
        die "Failed to download SHA256SUMS.txt."
    fi

    expected=$(awk -v name="$asset_name" '$2 == name { print $1; exit }' "$sums_path")
    if [ -z "$expected" ]; then
        die "SHA256SUMS.txt does not include $asset_name."
    fi

    actual=$(sha256_file "$archive_path")
    if [ "$actual" != "$expected" ]; then
        die "Checksum verification failed for $asset_name."
    fi

    info "Checksum verified."
}

choose_install_dir() {
    if [ -n "$INSTALL_DIR" ]; then
        printf '%s\n' "$INSTALL_DIR"
        return
    fi

    if [ -n "${HOME:-}" ] && mkdir -p "$HOME/.local/bin" 2>/dev/null && [ -w "$HOME/.local/bin" ]; then
        printf '%s\n' "$HOME/.local/bin"
        return
    fi

    printf '%s\n' "/usr/local/bin"
}

install_binary() {
    source_path=$1
    install_dir=$2
    target_path="${install_dir}/iqos"

    if mkdir -p "$install_dir" 2>/dev/null &&
        cp "$source_path" "$target_path" 2>/dev/null &&
        chmod 0755 "$target_path" 2>/dev/null; then
        return
    fi

    if ! has_cmd sudo; then
        die "Cannot write to $install_dir and sudo is not available. Set IQOS_CLI_INSTALL_DIR to a writable directory."
    fi

    info "Installing with sudo to $install_dir."
    sudo mkdir -p "$install_dir"
    sudo cp "$source_path" "$target_path"
    sudo chmod 0755 "$target_path"
}

cleanup() {
    if [ -n "${TMP_DIR:-}" ] && [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
}

main() {
    need_cmd uname
    need_cmd tar
    need_cmd mktemp

    requested_tag=$(normalize_version)
    api_url=$(release_api_url "$requested_tag")

    info "Fetching release metadata..."
    if ! release_json=$(http_get "$api_url"); then
        die "Failed to fetch release metadata from GitHub."
    fi

    tag=$(printf '%s' "$release_json" | json_string_value "tag_name")
    if [ -z "$tag" ]; then
        die "Could not read tag_name from GitHub release metadata."
    fi

    selected=$(select_asset "$release_json" "$tag")
    package=$(printf '%s\n' "$selected" | sed -n '1p')
    asset=$(printf '%s\n' "$selected" | sed -n '2p')
    asset_url=$(printf '%s\n' "$selected" | sed -n '3p')
    if [ -z "$package" ] || [ -z "$asset" ] || [ -z "$asset_url" ]; then
        die "Could not parse the selected release asset."
    fi

    TMP_DIR=$(mktemp -d)
    trap cleanup EXIT HUP INT TERM

    archive_path="${TMP_DIR}/${asset}"

    info "Installing IQOS CLI ${tag} (${package})."
    info "Downloading ${asset}..."
    if ! http_download "$asset_url" "$archive_path"; then
        die "Failed to download $asset."
    fi

    verify_checksum "$release_json" "$asset" "$archive_path" "$TMP_DIR"

    info "Extracting package..."
    archive_listing="${TMP_DIR}/archive.list"
    tar -tzf "$archive_path" > "$archive_listing"
    if grep -Eq '(^/|(^|/)\.\.(/|$))' "$archive_listing"; then
        die "The release archive contains unsafe paths."
    fi

    expected_member="iqos_cli-${tag}-${package}/iqos"
    if grep -Fx "$expected_member" "$archive_listing" >/dev/null; then
        binary_member="$expected_member"
    elif grep -Fx "./$expected_member" "$archive_listing" >/dev/null; then
        binary_member="./$expected_member"
    else
        die "The release archive did not contain an iqos binary."
    fi

    tar -xzf "$archive_path" -C "$TMP_DIR" "$binary_member"
    binary_path="${TMP_DIR}/${binary_member#./}"

    install_dir=$(choose_install_dir)
    info "Installing iqos to ${install_dir}..."
    install_binary "$binary_path" "$install_dir"

    info "Installation complete. Run 'iqos --version' to verify it."

    case ":${PATH:-}:" in
        *":${install_dir}:"*) ;;
        *)
            warn "${install_dir} is not in PATH. Add this to your shell profile:"
            warn "  export PATH=\"${install_dir}:\$PATH\""
            ;;
    esac
}

main "$@"
