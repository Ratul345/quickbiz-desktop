; QuickBiz for Windows: installer for the website download (per-user, no admin rights needed).
; Installs into %LOCALAPPDATA%\Programs\QuickBiz, a folder the app can write to, so in-app updates work.
; Build: iscc /DAppVersion=0.1.0 /DExePath=..\target\release\quickbiz-desktop.exe quickbiz.iss

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#ifndef ExePath
  #define ExePath "..\target\release\quickbiz-desktop.exe"
#endif

[Setup]
AppId={{6E0C8E4B-3B5E-4C57-9B71-5A1D2F7C9A10}
AppName=QuickBiz
AppVersion={#AppVersion}
AppPublisher=InfoSpark Labs
AppPublisherURL=https://quickbiz.vercel.app
AppSupportURL=https://quickbiz.vercel.app/contact
DefaultDirName={autopf}\QuickBiz
DefaultGroupName=QuickBiz
PrivilegesRequired=lowest
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=QuickBiz-Setup-{#AppVersion}
SetupIconFile=..\assets\images\icon.ico
UninstallDisplayIcon={app}\QuickBiz.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no

[Files]
Source: "{#ExePath}"; DestDir: "{app}"; DestName: "QuickBiz.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\QuickBiz"; Filename: "{app}\QuickBiz.exe"
Name: "{autodesktop}\QuickBiz"; Filename: "{app}\QuickBiz.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Run]
Filename: "{app}\QuickBiz.exe"; Description: "Open QuickBiz"; Flags: nowait postinstall skipifsilent
