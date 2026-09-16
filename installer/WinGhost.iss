#ifndef AppVersion
  #define AppVersion "0.2.0"
#endif
#ifndef SourceExe
  #define SourceExe "..\target\release\winghost.exe"
#endif

[Setup]
AppId={{D0C8D7CA-0B92-4D1E-93F1-7425189DC331}
AppName=WinGhost
AppVersion={#AppVersion}
AppPublisher=WinGhost contributors
AppPublisherURL=https://github.com/Ravshan04/WinGhost
AppSupportURL=https://github.com/Ravshan04/WinGhost/issues
DefaultDirName={autopf}\WinGhost
DefaultGroupName=WinGhost
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=WinGhost-Setup-x64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\WinGhost.exe

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "WinGhost.exe"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\WinGhost"; Filename: "{app}\WinGhost.exe"
Name: "{autodesktop}\WinGhost"; Filename: "{app}\WinGhost.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Run]
Filename: "{app}\WinGhost.exe"; Description: "Launch WinGhost"; Flags: nowait postinstall skipifsilent
