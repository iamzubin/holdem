; Holdem installer hooks (Tauri NSIS).
; Adds "Holdem" to Explorer's Send to menu, so files can be put on the shelf
; from a right-click; removed again on uninstall.

!macro NSIS_HOOK_POSTINSTALL
  CreateShortCut "$SENDTO\Holdem.lnk" "$INSTDIR\holdem.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$SENDTO\Holdem.lnk"
!macroend
