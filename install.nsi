!include "MUI2.nsh"

!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
  
!insertmacro MUI_LANGUAGE "English"

Name "Green Updater"
OutFile "green-installer.exe"
RequestExecutionLevel user

InstallDir "$DOCUMENTS\Green Updater"

Section
SetOutPath $INSTDIR

File /r dist
CreateShortcut "$SMPROGRAMS\Green Updater.lnk" "$INSTDIR\dist\green-updater.exe"

WriteUninstaller $INSTDIR\uninstaller.exe
SectionEnd

Section "Uninstall"
Delete "$SMPROGRAMS\Green Updater.lnk"

RMDir /r $INSTDIR\dist
Delete $INSTDIR\uninstaller.exe

RMDir $INSTDIR
SectionEnd
