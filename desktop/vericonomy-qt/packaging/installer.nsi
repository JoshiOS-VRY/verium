; Minimal NSIS installer for the Qt desktop wallet.
; Run after windeployqt has populated ..\dist with the binary + Qt runtime.

!define APPNAME "Vericonomy Wallet"
!define COMPANY "Vericonomy"
!define VERSION "0.1.0"

Name "${APPNAME}"
OutFile "..\dist\VericonomyWalletSetup.exe"
InstallDir "$PROGRAMFILES64\${APPNAME}"
RequestExecutionLevel admin

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
    SetOutPath "$INSTDIR"
    File /r "..\dist\*.*"
    CreateShortcut "$SMPROGRAMS\${APPNAME}.lnk" "$INSTDIR\verium-qt.exe"
    WriteUninstaller "$INSTDIR\uninstall.exe"
    ; Register the vericonomy:// deep-link scheme (handled by the host layer).
    WriteRegStr HKCR "vericonomy" "" "URL:Vericonomy Protocol"
    WriteRegStr HKCR "vericonomy" "URL Protocol" ""
    WriteRegStr HKCR "vericonomy\shell\open\command" "" '"$INSTDIR\verium-qt.exe" "%1"'
SectionEnd

Section "Uninstall"
    Delete "$SMPROGRAMS\${APPNAME}.lnk"
    DeleteRegKey HKCR "vericonomy"
    RMDir /r "$INSTDIR"
SectionEnd
