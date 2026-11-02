!macro NSIS_HOOK_POSTINSTALL
  Delete "$SMPROGRAMS\College Wi-Fi\Connect to College Wi-Fi.lnk"
  RMDir "$SMPROGRAMS\College Wi-Fi"
  CreateDirectory "$SMPROGRAMS\Wi-Fi Connect"
  CreateShortCut "$SMPROGRAMS\Wi-Fi Connect\Connect.lnk" "$INSTDIR\wifi-desktop.exe" "--connect" "$INSTDIR\wifi-desktop.exe"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$SMPROGRAMS\Wi-Fi Connect\Connect.lnk"
  RMDir "$SMPROGRAMS\Wi-Fi Connect"
  Delete "$SMPROGRAMS\College Wi-Fi\Connect to College Wi-Fi.lnk"
  RMDir "$SMPROGRAMS\College Wi-Fi"
!macroend
