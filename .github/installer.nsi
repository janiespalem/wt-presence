!include "MUI2.nsh"
!cd ".."

!ifndef VERSION
  !define VERSION "dev"
!endif

Name "WT Presence"
OutFile "dist\WT-Presence-Setup.exe"
InstallDir "$LOCALAPPDATA\WT Presence"
RequestExecutionLevel user
Unicode True

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "WT Presence" Main
  SetOutPath "$INSTDIR"
  File "target\release\wt-presence.exe"
  File "README.md"
  File "LICENSE"
  SetOutPath "$INSTDIR\web"
  File /r "web\dist\*"
  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\WT Presence"
  CreateShortcut "$SMPROGRAMS\WT Presence\WT Presence.lnk" "$INSTDIR\wt-presence.exe"
  CreateShortcut "$SMPROGRAMS\WT Presence\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\WT Presence\WT Presence.lnk"
  Delete "$SMPROGRAMS\WT Presence\Uninstall.lnk"
  RMDir "$SMPROGRAMS\WT Presence"
  RMDir /r "$INSTDIR\web"
  Delete "$INSTDIR\wt-presence.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
