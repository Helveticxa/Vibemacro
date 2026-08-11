#define MyAppName "VibeTimer"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Helvetica"
#define MyAppExeName "VibeTimer.exe"

[Setup]
AppId={{F677B6B9-347D-4D9F-9444-23A7DA9C6822}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppComments=Timer dan macro Windows ringan untuk alur kerja AI.
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
OutputDir=..\dist
OutputBaseFilename=VibeTimer-Setup-{#MyAppVersion}-x64
SetupIconFile=..\assets\VibeTimer.ico
UninstallDisplayIcon={app}\VibeTimer.ico
LicenseFile=..\LICENSE
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern dark polar includetitlebar
WizardSizePercent=110
CloseApplications=yes
RestartApplications=no
AppMutex=Local\VibeTimer.SingleInstance.v1
SetupLogging=yes
VersionInfoVersion=1.0.0.0
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=Installer {#MyAppName}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}
VersionInfoCopyright=Copyright (c) 2026 {#MyAppPublisher}

[Tasks]
Name: "desktopicon"; Description: "Buat shortcut di Desktop"; GroupDescription: "Shortcut tambahan:"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\VibeTimer.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\VibeTimer.ico"; Comment: "Buka VibeTimer"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\VibeTimer.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Jalankan {#MyAppName}"; Flags: nowait postinstall skipifsilent

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  AutoStartCommand: String;
  ExpectedCommand: String;
begin
  if CurUninstallStep = usUninstall then begin
    ExpectedCommand := '"' + ExpandConstant('{app}\VibeTimer.exe') + '" --background';
    if RegQueryStringValue(
      HKCU,
      'Software\Microsoft\Windows\CurrentVersion\Run',
      'VibeTimer',
      AutoStartCommand
    ) and (CompareText(AutoStartCommand, ExpectedCommand) = 0) then
      RegDeleteValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Run', 'VibeTimer');
  end;
end;
