OutFile "green-installer.exe"

InstallDir $DESKTOP\test

Section
SetOutPath $INSTDIR

File dist

WriteUninstaller $INSTDIR\uninstaller.exe
SectionEnd

Section "Uninstall"
Delete $INSTDIR\dist
Delete $INSTDIR\uninstaller.exe

RMDir $INSTDIR
SectionEnd
