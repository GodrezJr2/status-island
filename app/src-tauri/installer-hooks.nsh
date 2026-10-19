; Status Island NSIS hooks (per-user install, no admin).

!macro NSIS_HOOK_POSTINSTALL
  ; VS Code's claudeCode.claudeProcessWrapper takes an exe path only; this copy's
  ; file name selects wrap mode (AGENTS.md, "Wrapper launch").
  CopyFiles /SILENT "$INSTDIR\status-island.exe" "$INSTDIR\si-wrap.exe"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Put back every settings file Status Island changed, then remove its data folder.
  ExecWait '"$INSTDIR\status-island.exe" --uninstall-cleanup'
  Delete "$INSTDIR\si-wrap.exe"
!macroend
