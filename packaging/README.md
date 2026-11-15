# Packaging QuickBiz for Windows

| Output | For | Tool | Updates |
|---|---|---|---|
| `dist/QuickBiz-Setup-<version>.exe` | the website download | Inno Setup 6 | in-app (writable per-user folder) |
| `dist/QuickBiz-<version>.0.msix` | Microsoft Store | Windows 10/11 SDK (`makeappx`) | the Store (`--features store` build) |

Run `./build.ps1 -Target All` from this folder. Output goes to `dist/` (not committed).

## Before the Store upload
1. Reserve the app name in Partner Center, then open *Product identity*.
2. Run the build with `-IdentityName`, `-Publisher` (the `CN=...` string) and `-PublisherDisplayName`
   copied exactly from that page. The package is rejected if they differ.
3. Upload the unsigned `.msix`; Microsoft signs it.
4. Run the Windows App Certification Kit on the package first.

## Website installer
Unsigned installers show a SmartScreen warning until they build reputation or are signed with a
code-signing certificate. Upload the installer, then set `windowsDownloadUrl` / `windowsVersion` in `settings`.
