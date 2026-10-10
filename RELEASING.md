# Release process

Starting with v1.1.0, `main` builds the only supported executable,
`bad_apple.exe`. The former `rewrite` branch and separate `bad_apple_win`
project are historical. Both visual modes ship in one Windows x64 package.

## Validate and package

On Windows with Rust stable and the MSVC/Windows SDK build tools:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
.\scripts\package.ps1
```

The script reads the package version from Cargo, creates
`target/dist/bad-apple-windows-v<VERSION>.zip`, and generates `SHA256SUMS`
for the executable and ZIP. The ZIP contains the executable, two launchers,
README, LICENSE, and its executable checksum. Every release must include
`SHA256SUMS`; repackage if the binary changes.

When an audio device is available, also run the zero-volume integration probes:

```powershell
cargo run --locked --release --example audio_probe -- 12
cargo run --locked --release --example pause_probe
```

Manually verify both modes, Ctrl+Alt+F8 pause/resume, Ctrl+Alt+F9 exit,
and shortcut release after exit. CI does not require an audio device.

## Publish with GitHub CLI

Commit and push the validated sources first. Create the release against that
exact commit, with a tag matching Cargo.toml. For example:

```powershell
gh auth setup-git
git push origin main
gh release create v1.1.0 --repo blycr/bad_apple_virus --target (git rev-parse HEAD) `
  --title 'Bad Apple Windows Player v1.1.0' --notes-file RELEASE_NOTES.md `
  target\release\bad_apple.exe target\dist\bad-apple-windows-v1.1.0.zip target\dist\SHA256SUMS
```

Verify the uploaded filenames, sizes, and SHA-256 digests using the GitHub
release-assets API, and check that the tag resolves to the committed sources.

Users can verify an executable or ZIP with:

```powershell
Get-FileHash .\bad_apple.exe -Algorithm SHA256
Get-FileHash .\bad-apple-windows-v1.1.0.zip -Algorithm SHA256
```

Compare with `SHA256SUMS`. Linux/macOS can use `sha256sum -c SHA256SUMS`
when both files are present. GitHub Actions also packages each successful build
as a workflow artifact; publishing a release is a separate, explicit step.
