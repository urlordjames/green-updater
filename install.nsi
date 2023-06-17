!include "MUI2.nsh"

!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Name "Green Updater 2"
OutFile "green-installer.exe"

InstallDir "$PROGRAMFILES64\Green Updater"

Section
SetOutPath $INSTDIR

CreateDirectory $INSTDIR\bin
File /oname=bin\green-updater.exe target\release\green-updater.exe
CreateShortcut "$SMPROGRAMS\Green Updater 2.lnk" "$INSTDIR\bin\green-updater.exe"

WriteUninstaller $INSTDIR\uninstaller.exe

WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GreenUpdater" \
	"DisplayName" "Green Updater 2"

WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GreenUpdater" \
	"UninstallString" "$\"$INSTDIR\uninstaller.exe$\""
SectionEnd

Section "Uninstall"
Delete "$SMPROGRAMS\Green Updater 2.lnk"
Delete $INSTDIR\bin\green-updater.exe
RMDir $INSTDIR\bin

Delete $INSTDIR\uninstaller.exe

DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\GreenUpdater"

RMDir $INSTDIR
SectionEnd
