# Release checklist

How to cut a new release for this repository. Both branches (`main` = the
original project, `rewrite` = the self-contained rewrite) follow the same
process.

## Mandatory: checksum file

**Every release MUST ship a `SHA256SUMS` file** with the SHA-256 checksum of
every executable asset. Without it, users have no way to verify that a
downloaded binary was not corrupted or tampered with.

```powershell
# build first
cargo build --release

# generate the checksum file (lowercase, two spaces, filename as uploaded)
(Get-FileHash target\release\<name>.exe -Algorithm SHA256).Hash.ToLower() |
  ForEach-Object { "$_  <name>.exe" } | Set-Content SHA256SUMS -Encoding ascii

# create the release with both the binary and the checksum file
gh release create <tag> --title "<title>" --notes-file NOTES.md `
  target\release\<name>.exe SHA256SUMS
```

Then update the release notes to print the checksum and how to verify:

- Windows (PowerShell): `Get-FileHash <name>.exe -Algorithm SHA256`
- Linux/macOS: `sha256sum -c SHA256SUMS`

## Notes for this machine

- **Antivirus quarantines downloads to temp directories.** On this machine
  火绒/Huorong deletes `bad_apple.exe` (the original's binary) shortly after
  it is downloaded into a temp folder, and it can also flag it when run. Do
  not burn time on download-then-verify loops for that binary; the checksum
  is computed from the freshly built artifact at upload time. Warn users in
  the release notes to whitelist the binary.
- Re-verify the uploaded asset's SHA-256 matches the local build before
  publishing the checksum file (`gh release download` + `Get-FileHash`), or
  simply trust the build artifact that was uploaded in the same command.

## Tag naming

- Original (main): `v1.0.0`, `v1.0.1`, ... (or `v<major>.<minor>.<patch>`)
- Rewrite (rewrite branch): `v1.0.0-rewrite`, ... (suffix `-rewrite`)
