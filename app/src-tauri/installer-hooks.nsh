; Status Island NSIS hooks (per-user install, no admin).

!macro NSIS_HOOK_POSTINSTALL
  ; VS Code's claudeCode.claudeProcessWrapper takes an exe path only; this copy's
  ; file name selects wrap mode (AGENTS.md, "Wrapper launch").
  ; Claude sessions may be running through the old copy during an update. A running exe
  ; can't be overwritten but can be renamed, so move it aside first; the next install removes it.
  Delete "$INSTDIR\si-wrap.old.exe"
  Rename "$INSTDIR\si-wrap.exe" "$INSTDIR\si-wrap.old.exe"
  CopyFiles /SILENT "$INSTDIR\status-island.exe" "$INSTDIR\si-wrap.exe"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; An update (/UPDATE) replaces the app in place: keep the connector and settings as they are.
  ${If} $UpdateMode <> 1
    ; Put back every settings file Status Island changed, then remove its data folder.
    ExecWait '"$INSTDIR\status-island.exe" --uninstall-cleanup' $0
    IntCmp $0 0 +2
      Abort "Status Island could not restore your settings. Uninstall stopped so the backup remains available."
    Delete "$INSTDIR\si-wrap.exe"
    Delete /REBOOTOK "$INSTDIR\si-wrap.old.exe"
  ${EndIf}
!macroend
