; Horizon Offline Launcher's Windows installer (Inno Setup 6), built by .github/workflows/launcher.yml:
;   iscc /DAppVersion=1.2.0 /DSource=path\to\horizon-launcher.exe /Odist packaging\windows\setup.iss
; It installs for this user only, without administrator rights, in the same place as install.ps1
; (%LOCALAPPDATA%\Programs\HorizonOfflineLauncher), so either one updates the other's install and the
; launcher's own updates replace the same file. Uninstalling keeps the game, worlds and settings
; (%APPDATA%\horizon-launcher).

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef Source
  #define Source "..\..\launcher\target\release\horizon-launcher.exe"
#endif
#define AppName "Horizon Offline Launcher"
#define AppExe "horizon-offline-launcher.exe"

[Setup]
AppId={{6F0C3B7E-4A1D-4C8B-9E52-3D7A1F2B8C90}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=gerdaroot
AppPublisherURL=https://github.com/gerdaroot/Horizon-Offline-Launcher
AppSupportURL=https://github.com/gerdaroot/Horizon-Offline-Launcher/issues
AppUpdatesURL=https://github.com/gerdaroot/Horizon-Offline-Launcher/releases
DefaultDirName={localappdata}\Programs\HorizonOfflineLauncher
DisableDirPage=yes
DisableProgramGroupPage=yes
DisableReadyPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputBaseFilename=Horizon-Offline-Launcher-Setup
SetupIconFile=..\icons\horizon.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
LicenseFile=..\..\LICENSE
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
CloseApplications=force

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#Source}"; DestDir: "{app}"; DestName: "{#AppExe}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"
Name: "{userdesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
