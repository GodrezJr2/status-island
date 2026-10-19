# winget

The manifest needs the released installer's URL and SHA-256, so it's created **after** a release exists, not before.

```powershell
winget install wingetcreate
wingetcreate new "https://github.com/Hossain-13/status-island/releases/download/v0.1.0/Status.Island_0.1.0_x64-setup.exe" "https://github.com/Hossain-13/status-island/releases/download/v0.1.0/Status.Island_0.1.0_arm64-setup.exe"
```

Use these values when prompted:
- **PackageIdentifier:** `Hossain-13.StatusIsland`
- **Publisher:** `Sheikh Mohammad Hossain`
- **License:** `MIT`
- **InstallerType:** `nullsoft`
- **Scope:** `user`

Commit the generated manifests to this folder. Then submit them with `wingetcreate submit` (it opens a pull request to microsoft/winget-pkgs). Copy the exact asset URLs from the release page, because GitHub replaces the space in "Status Island" with a dot.
