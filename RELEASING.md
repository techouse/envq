# Releasing

Use this runbook before creating a public envq release.

## Local Gate

Run the full maintainer gate:

```bash
make pre-release
```

The required GitHub Actions workflow must pass on Linux, macOS, and Windows
before a release tag is pushed.

## Release Dry Run

Use the manual `Release Dry Run` workflow before pushing a release tag. The
default dry run validates package metadata, builds Linux and Windows artifacts,
generates `SHA256SUMS.txt` plus per-artifact `.sha256` sidecars, creates GitHub
Artifact Attestations, and uploads combined dry-run artifacts.

Dry-run artifacts are retained for 1 day. The workflow does not publish to
crates.io, create a tag, or create a GitHub Release.

The dry-run artifact version label defaults to `nightly` so manual validation
artifacts cannot be confused with a real release version. Override the `version`
input when validating exact release asset names. Linux and Windows dry-run
builds are enabled by default through `run-linux` and `run-windows`. Signed and
notarized macOS dry runs are opt-in. Enable `run-macos-signed` only when the
`release` GitHub environment and Apple notarization service should be used.

## Manual Smoke Test

Create a temporary file and verify only expected bytes change:

```bash
tmp="$(mktemp -d)"
env_file="$tmp/demo.env"
printf 'A=1\nB=two\n' > "$env_file"

target/release/envq get A "$env_file"
target/release/envq list "$env_file"
target/release/envq set A 2 "$env_file" --diff
target/release/envq set A 2 "$env_file" --check
target/release/envq set A 2 "$env_file"
target/release/envq unset B "$env_file" --stdout
```

Generate every supported completion script:

```bash
target/release/envq completion bash >/tmp/envq.bash
target/release/envq completion zsh >/tmp/_envq
target/release/envq completion fish >/tmp/envq.fish
target/release/envq completion powershell >/tmp/envq.ps1
target/release/envq completion pwsh >/tmp/envq-pwsh.ps1
```

## Tag And Publish

1. Confirm `Cargo.toml` and `CHANGELOG.md` contain the release version.
2. Commit all release changes.
3. Create and push the version tag:

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. The tag-triggered release workflow builds Linux GNU/glibc `.tar.gz`, `.deb`,
   and `.rpm` artifacts plus musl `.tar.gz` artifacts for x86_64 and ARM64,
   builds a Windows MSVC archive, builds a signed and notarized universal macOS
   archive, publishes to crates.io after those builds pass, generates
   `SHA256SUMS.txt` plus per-artifact `.sha256` sidecars, and creates the
   GitHub Release.

## Release Provenance

The release workflow creates GitHub Artifact Attestations for every final
release file, including Linux `.tar.gz`, `.deb`, and `.rpm` artifacts, the
Windows `.zip`, the macOS `.zip`, per-artifact `.sha256` sidecars, and
`SHA256SUMS.txt`.

Verify a downloaded artifact with:

```bash
gh attestation verify <artifact> -R techouse/envq
```

Artifact attestations provide build provenance and integrity for release files.
They do not replace macOS Developer ID signing and notarization, Windows
Authenticode signing, or future Linux package repository signing.

## Checksum Verification

Every primary release artifact has a sibling `.sha256` file. The release also
includes an aggregate `SHA256SUMS.txt` for verifying all downloaded artifacts at
once. The aggregate checksum file lists only primary artifacts, not `.sha256`
sidecars or `SHA256SUMS.txt` itself.

Verify all downloaded artifacts from a release directory with:

```bash
sha256sum -c SHA256SUMS.txt
```

Verify one artifact with its sidecar:

```bash
sha256sum -c envq-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```

## Linux Release Artifacts

Linux releases are built natively on GitHub-hosted runners for:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`

Each GNU/glibc architecture produces:

- `.tar.gz` archive
- `.deb` package
- `.rpm` package

Each musl architecture produces:

- `.tar.gz` archive

Musl archives are intended for Alpine Linux, small container images, and other
portable binary use cases. `.deb` and `.rpm` packages remain GNU/glibc-only so
the distro packages match the target libc expected by those ecosystems.

The `.deb` and `.rpm` packages install `envq` to `/usr/bin/envq` and install
bash, zsh, and fish completions to the standard distro completion paths.
PowerShell completions remain generated on demand with
`envq completion powershell` or `envq completion pwsh`.

## Windows Release Artifact

Windows releases are built for `x86_64-pc-windows-msvc` and packaged as a `.zip`
archive containing:

- `envq.exe`
- bash, zsh, fish, and PowerShell completion files under `completions/`
- `README.md`
- `LICENSE`

The Windows binary is currently unsigned. Add Windows code signing before the
first public binary release once the signing provider is chosen.

## macOS Release Artifact

The macOS release artifact is a universal Mach-O binary containing both
`x86_64` and `arm64` slices. The minimum supported macOS version for this
artifact is macOS 11.0.

Crates.io publishing, GitHub Release creation, and signed macOS release jobs use
the `release` GitHub environment. Configure that environment with required
reviewers if manual approval should be required before publishing or signing. It
must provide this crates.io secret:

- `CARGO_REGISTRY_TOKEN`

It must also provide these Apple signing and notarization secrets:

- `APPLE_ID`
- `BUILD_CERTIFICATE_BASE64`
- `BUILD_CERTIFICATE_SHA1`
- `KEYCHAIN_PASSWORD`
- `NOTARYTOOL_PASSWORD`
- `P12_PASSWORD`
- `TEAM_ID`

It must also provide this environment variable:

- `NOTARYTOOL_KEYCHAIN_PROFILE`

Manual fallback, only if the release workflow is unavailable:

```bash
cargo publish --locked
cargo build --release --locked
```

Then archive `target/release/envq` or `target/release/envq.exe` with
`README.md` and `LICENSE`, generate `SHA256SUMS.txt` plus per-artifact
`.sha256` sidecars, and upload the files to the GitHub Release.

Manual Linux packaging fallback, only if the Linux release workflow is
unavailable:

```bash
cargo install cargo-deb cargo-generate-rpm
cargo build --release --locked
mkdir -p target/completions dist
target/release/envq completion bash > target/completions/envq.bash
target/release/envq completion zsh > target/completions/_envq
target/release/envq completion fish > target/completions/envq.fish
package="envq-0.1.0-$(rustc -vV | awk '/host:/ {print $2}')"
mkdir -p "dist/$package"
cp target/release/envq "dist/$package/envq"
cp README.md LICENSE "dist/$package/"
tar -czf "dist/$package.tar.gz" -C dist "$package"
cargo deb --no-build
cargo generate-rpm
cp target/debian/*.deb target/generate-rpm/*.rpm dist/
case "$(rustc -vV | awk '/host:/ {print $2}')" in
  x86_64-unknown-linux-gnu)
    musl_target="x86_64-unknown-linux-musl"
    ;;
  aarch64-unknown-linux-gnu)
    musl_target="aarch64-unknown-linux-musl"
    ;;
  *)
    musl_target=""
    ;;
esac
if [ -n "$musl_target" ]; then
  sudo apt-get update
  sudo apt-get install -y musl-tools
  rustup target add "$musl_target"
  cargo build --release --locked --target "$musl_target"
  musl_package="envq-0.1.0-$musl_target"
  mkdir -p "dist/$musl_package"
  cp "target/$musl_target/release/envq" "dist/$musl_package/envq"
  cp README.md LICENSE "dist/$musl_package/"
  tar -czf "dist/$musl_package.tar.gz" -C dist "$musl_package"
fi
```

Manual Windows packaging fallback, only if the Windows release workflow is
unavailable:

```powershell
cargo build --release --locked --target x86_64-pc-windows-msvc
$package = "envq-0.1.0-x86_64-pc-windows-msvc"
New-Item -ItemType Directory -Force "dist\$package\completions" | Out-Null
target\x86_64-pc-windows-msvc\release\envq.exe completion bash > "dist\$package\completions\envq.bash"
target\x86_64-pc-windows-msvc\release\envq.exe completion zsh > "dist\$package\completions\_envq"
target\x86_64-pc-windows-msvc\release\envq.exe completion fish > "dist\$package\completions\envq.fish"
target\x86_64-pc-windows-msvc\release\envq.exe completion powershell > "dist\$package\completions\envq.ps1"
Copy-Item target\x86_64-pc-windows-msvc\release\envq.exe "dist\$package\envq.exe"
Copy-Item README.md, LICENSE "dist\$package"
Compress-Archive -Path "dist\$package\*" -DestinationPath "dist\$package.zip"
```

Manual macOS fallback, only if the macOS release workflow is unavailable:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release --locked --target aarch64-apple-darwin
MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release --locked --target x86_64-apple-darwin
mkdir -p dist
lipo -create \
  target/aarch64-apple-darwin/release/envq \
  target/x86_64-apple-darwin/release/envq \
  -output dist/envq
codesign \
  --force \
  --sign "$BUILD_CERTIFICATE_SHA1" \
  --identifier com.techouse.envq \
  --options runtime \
  --timestamp \
  dist/envq
package="envq-0.1.0-universal-apple-darwin"
mkdir -p "dist/$package"
cp dist/envq "dist/$package/envq"
cp README.md LICENSE "dist/$package/"
ditto -c -k --keepParent "dist/$package" "dist/$package.zip"
xcrun notarytool submit "dist/$package.zip" \
  --keychain-profile "$NOTARYTOOL_KEYCHAIN_PROFILE" \
  --wait
```

## Checklist Mapping

- Cross-platform correctness: required CI, binary path tests, CRLF/mixed-newline
  fixtures, and atomic-write tests.
- CLI UX and exit codes: unit tests and golden fixtures.
- Behavior guarantees: golden parse/edit/CLI fixtures, duplicate-key tests,
  idempotency tests, large-file tests, and fuzz targets.
- Binary distribution: tag-triggered GitHub Release archives, Linux `.deb` and
  `.rpm` packages, Linux musl tarballs, signed/notarized universal macOS
  artifact, checksums, and GitHub Artifact Attestations.
- Packaging: `cargo package`, `cargo publish --dry-run`, and automated
  tag-triggered `cargo publish`.
- Shell integration: CI smoke tests for bash, zsh, fish, PowerShell, and pwsh.
- Security: no shell execution, same-directory temporary files, and path handling
  through `Path`/`PathBuf`.
