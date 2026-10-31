!macro NSIS_HOOK_POSTINSTALL
  CreateDirectory "$SMPROGRAMS\College Wi-Fi"
  CreateShortCut "$SMPROGRAMS\College Wi-Fi\Connect to College Wi-Fi.lnk" "$INSTDIR\wifi-desktop.exe" "--connect" "$INSTDIR\wifi-desktop.exe"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$SMPROGRAMS\College Wi-Fi\Connect to College Wi-Fi.lnk"
  RMDir "$SMPROGRAMS\College Wi-Fi"
!macroend
