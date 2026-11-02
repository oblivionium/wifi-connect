!include "StrFunc.nsh"
${StrTok}
${UnStrTok}

!macro NSIS_HOOK_POSTINSTALL
  Delete "$SMPROGRAMS\College Wi-Fi\Connect to College Wi-Fi.lnk"
  RMDir "$SMPROGRAMS\College Wi-Fi"
  CreateDirectory "$SMPROGRAMS\Wi-Fi Connect"
  CreateShortCut "$SMPROGRAMS\Wi-Fi Connect\Connect.lnk" "$INSTDIR\wifi-desktop.exe" "--connect" "$INSTDIR\wifi-desktop.exe"

  ReadRegStr $0 HKCU "Environment" "Path"
  StrCpy $1 0
wifi_cli_path_scan:
  ${StrTok} $2 "$0" ";" "$1" "1"
  StrCmp $2 "" wifi_cli_add_to_path
  StrCmp $2 "$INSTDIR" wifi_cli_path_updated
  IntOp $1 $1 + 1
  Goto wifi_cli_path_scan
wifi_cli_add_to_path:
  StrCmp $0 "" wifi_cli_path_empty
  WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR"
  Goto wifi_cli_path_updated
wifi_cli_path_empty:
  WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
wifi_cli_path_updated:
  System::Call 'user32::SendMessageTimeoutW(p 0xffff, i 0x001A, p 0, w "Environment", i 0x0002, i 5000, *p .r0)'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ReadRegStr $0 HKCU "Environment" "Path"
  StrCpy $1 ""
  StrCpy $2 0
wifi_cli_path_remove_scan:
  ${UnStrTok} $3 "$0" ";" "$2" "1"
  StrCmp $3 "" wifi_cli_path_remove_done
  StrCmp $3 "$INSTDIR" wifi_cli_path_remove_next
  StrCmp $1 "" wifi_cli_path_remove_first
  StrCpy $1 "$1;$3"
  Goto wifi_cli_path_remove_next
wifi_cli_path_remove_first:
  StrCpy $1 "$3"
wifi_cli_path_remove_next:
  IntOp $2 $2 + 1
  Goto wifi_cli_path_remove_scan
wifi_cli_path_remove_done:
  StrCmp $1 "" wifi_cli_delete_user_path
  WriteRegExpandStr HKCU "Environment" "Path" "$1"
  Goto wifi_cli_user_path_updated
wifi_cli_delete_user_path:
  DeleteRegValue HKCU "Environment" "Path"
wifi_cli_user_path_updated:
  System::Call 'user32::SendMessageTimeoutW(p 0xffff, i 0x001A, p 0, w "Environment", i 0x0002, i 5000, *p .r0)'
  Delete "$SMPROGRAMS\Wi-Fi Connect\Connect.lnk"
  RMDir "$SMPROGRAMS\Wi-Fi Connect"
  Delete "$SMPROGRAMS\College Wi-Fi\Connect to College Wi-Fi.lnk"
  RMDir "$SMPROGRAMS\College Wi-Fi"
!macroend
