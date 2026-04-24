#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALLER="${ROOT_DIR}/install.sh"
SMOKE_ROOT="${ENVQ_INSTALLER_SMOKE_ROOT:-$(mktemp -d)}"
KEEP_SMOKE_ROOT="${ENVQ_KEEP_INSTALLER_SMOKE_ROOT:-}"
FAKEBIN="${SMOKE_ROOT}/fakebin"
DIST="${SMOKE_ROOT}/dist"
ZERO_SHA256="0000000000000000000000000000000000000000000000000000000000000000"

if [[ -z "$KEEP_SMOKE_ROOT" ]]; then
    trap 'rm -rf "$SMOKE_ROOT"' EXIT
else
    printf 'Keeping installer smoke root: %s\n' "$SMOKE_ROOT"
fi

mkdir -p "$FAKEBIN" "$DIST"

write_fake_tools() {
    cat >"${FAKEBIN}/curl" <<'FAKE_CURL'
#!/usr/bin/env sh
set -eu

out=""
url=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        -o)
            shift
            out="$1"
            ;;
        -*)
            ;;
        *)
            url="$1"
            ;;
    esac
    shift
done

case "$url" in
    *api.github.com*)
        if [ "${ENVQ_FAKE_API_FORBIDDEN:-}" = "1" ]; then
            printf 'unexpected GitHub API request\n' >&2
            exit 22
        fi
        printf '%s\n' '{"tag_name":"v9.9.9"}'
        ;;
    *)
        if [ -z "$out" ]; then
            printf 'missing curl -o destination for %s\n' "$url" >&2
            exit 2
        fi
        file="${url##*/}"
        cp "${ENVQ_FAKE_DIST:?}/${file}" "$out"
        ;;
esac
FAKE_CURL

    cat >"${FAKEBIN}/uname" <<'FAKE_UNAME'
#!/usr/bin/env sh
set -eu

case "${1:-}" in
    -s)
        printf '%s\n' "${ENVQ_FAKE_OS:?}"
        ;;
    -m)
        printf '%s\n' "${ENVQ_FAKE_ARCH:?}"
        ;;
    *)
        /usr/bin/uname "$@"
        ;;
esac
FAKE_UNAME

    cat >"${FAKEBIN}/ldd" <<'FAKE_LDD'
#!/usr/bin/env sh
set -eu

case "${ENVQ_FAKE_LIBC:-gnu}" in
    musl)
        printf 'musl libc\n'
        ;;
    *)
        printf 'ldd (GNU libc) 2.39\n'
        ;;
esac
FAKE_LDD

    chmod 755 "${FAKEBIN}/curl" "${FAKEBIN}/uname" "${FAKEBIN}/ldd"
}

write_checksum() {
    local archive="$1"

    if command -v sha256sum >/dev/null 2>&1; then
        (cd "$DIST" && sha256sum "$archive" >"${archive}.sha256")
    else
        (cd "$DIST" && shasum -a 256 "$archive" >"${archive}.sha256")
    fi
}

make_package() {
    local package="$1"
    local archive="$2"
    local package_dir="${DIST}/${package}"

    rm -rf "$package_dir"
    mkdir -p "$package_dir"

    cat >"${package_dir}/envq" <<'FAKE_ENVQ'
#!/usr/bin/env sh
set -eu

if [ "${1:-}" = "--version" ]; then
    printf 'envq 9.9.9\n'
else
    printf 'fake envq\n'
fi
FAKE_ENVQ
    chmod 755 "${package_dir}/envq"

    printf 'readme\n' >"${package_dir}/README.md"
    printf 'license\n' >"${package_dir}/LICENSE"
    printf 'third-party licenses\n' >"${package_dir}/THIRD-PARTY-LICENSES.md"

    rm -f "${DIST}/${archive}" "${DIST}/${archive}.sha256"
    case "$archive" in
        *.tar.gz)
            (cd "$DIST" && tar -czf "$archive" "$package")
            ;;
        *.zip)
            (cd "$DIST" && zip -qr "$archive" "$package")
            ;;
        *)
            printf 'unsupported fake archive: %s\n' "$archive" >&2
            exit 2
            ;;
    esac

    write_checksum "$archive"
}

run_install() {
    local name="$1"
    local os="$2"
    local arch="$3"
    local libc="$4"
    local expected_target="$5"
    local forced_linux_libc="${6:-}"
    local install_dir="${SMOKE_ROOT}/install/${name}"
    local output="${SMOKE_ROOT}/${name}.out"

    rm -rf "$install_dir"
    mkdir -p "$install_dir"

    if [[ -n "$forced_linux_libc" ]]; then
        if ! env \
            PATH="${FAKEBIN}:${PATH}" \
            ENVQ_FAKE_DIST="$DIST" \
            ENVQ_FAKE_OS="$os" \
            ENVQ_FAKE_ARCH="$arch" \
            ENVQ_FAKE_LIBC="$libc" \
            ENVQ_LINUX_LIBC="$forced_linux_libc" \
            ENVQ_INSTALL_DIR="$install_dir" \
            sh "$INSTALLER" >"$output" 2>&1; then
            cat "$output" >&2
            exit 1
        fi
    else
        if ! env \
            PATH="${FAKEBIN}:${PATH}" \
            ENVQ_FAKE_DIST="$DIST" \
            ENVQ_FAKE_OS="$os" \
            ENVQ_FAKE_ARCH="$arch" \
            ENVQ_FAKE_LIBC="$libc" \
            ENVQ_INSTALL_DIR="$install_dir" \
            sh "$INSTALLER" >"$output" 2>&1; then
            cat "$output" >&2
            exit 1
        fi
    fi

    "${install_dir}/envq" --version | grep -qx 'envq 9.9.9'
    grep -Fqx "[INFO] Target: ${expected_target}" "$output"
    printf 'ok: %s\n' "$name"
}

expect_install_failure() {
    local name="$1"
    local os="$2"
    local arch="$3"
    local libc="$4"
    local expected_error="$5"
    local output="${SMOKE_ROOT}/${name}.out"

    if env \
        PATH="${FAKEBIN}:${PATH}" \
        ENVQ_FAKE_DIST="$DIST" \
        ENVQ_FAKE_OS="$os" \
        ENVQ_FAKE_ARCH="$arch" \
        ENVQ_FAKE_LIBC="$libc" \
        ENVQ_INSTALL_DIR="${SMOKE_ROOT}/install/${name}" \
        sh "$INSTALLER" >"$output" 2>&1; then
        cat "$output" >&2
        printf 'expected installer failure for %s\n' "$name" >&2
        exit 1
    fi

    grep -Fq "$expected_error" "$output"
    printf 'ok: %s\n' "$name"
}

main() {
    write_fake_tools

    make_package \
        "envq-9.9.9-x86_64-unknown-linux-gnu" \
        "envq-9.9.9-x86_64-unknown-linux-gnu.tar.gz"
    make_package \
        "envq-9.9.9-x86_64-unknown-linux-musl" \
        "envq-9.9.9-x86_64-unknown-linux-musl.tar.gz"
    make_package \
        "envq-9.9.9-aarch64-unknown-linux-musl" \
        "envq-9.9.9-aarch64-unknown-linux-musl.tar.gz"
    make_package \
        "envq-9.9.9-universal-apple-darwin" \
        "envq-9.9.9-universal-apple-darwin.zip"

    run_install "linux-gnu-x86_64" \
        "Linux" "x86_64" "gnu" "x86_64-unknown-linux-gnu"
    run_install "linux-musl-aarch64" \
        "Linux" "aarch64" "musl" "aarch64-unknown-linux-musl"
    run_install "linux-musl-override" \
        "Linux" "x86_64" "gnu" "x86_64-unknown-linux-musl" "musl"
    run_install "macos-universal" \
        "Darwin" "arm64" "gnu" "universal-apple-darwin"

    printf '%s  %s\n' \
        "$ZERO_SHA256" \
        "envq-9.9.9-x86_64-unknown-linux-gnu.tar.gz" \
        >"${DIST}/envq-9.9.9-x86_64-unknown-linux-gnu.tar.gz.sha256"
    expect_install_failure "bad-checksum" \
        "Linux" "x86_64" "gnu" "Checksum verification failed"
    expect_install_failure "unsupported-arch" \
        "Linux" "s390x" "gnu" "Unsupported architecture"
    expect_install_failure "unsupported-os" \
        "FreeBSD" "x86_64" "gnu" "Unsupported operating system"
}

main "$@"
