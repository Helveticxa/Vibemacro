#define MyAppName "Vibemacro"
#define MyAppVersion "1.2.0"
#define MyAppPublisher "Helveticxa"
#define MyAppExeName "Vibemacro.exe"
#ifndef MyAppId
  #define MyAppId "{{F677B6B9-347D-4D9F-9444-23A7DA9C6822}"
#endif
#ifndef MyOutputDir
  #define MyOutputDir "..\dist"
#endif
#ifndef MyOutputBaseFilename
  #define MyOutputBaseFilename "Vibemacro-Setup-1.2.0-x64"
#endif

[Setup]
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppComments=Timer dan macro Windows-native yang ringan dan vendor-neutral.
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
OutputDir={#MyOutputDir}
OutputBaseFilename={#MyOutputBaseFilename}
SetupIconFile=..\assets\Vibemacro.ico
UninstallDisplayIcon={app}\Vibemacro.ico
LicenseFile=..\LICENSE
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern dark polar includetitlebar
WizardSizePercent=110
CloseApplications=yes
RestartApplications=no
AppMutex=Local\VibeTimer.SingleInstance.v1
SetupLogging=yes
VersionInfoVersion=1.2.0.0
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=Installer {#MyAppName}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}
VersionInfoCopyright=Copyright (c) 2026 {#MyAppPublisher}

[Tasks]
Name: "desktopicon"; Description: "Buat shortcut di Desktop"; GroupDescription: "Shortcut tambahan:"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\Vibemacro.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\Vibemacro.ico"; Comment: "Buka Vibemacro"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\Vibemacro.ico"; Tasks: desktopicon

[InstallDelete]
Type: files; Name: "{app}\VibeTimer.exe"
Type: files; Name: "{app}\VibeTimer.ico"
Type: files; Name: "{group}\VibeTimer.lnk"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Jalankan {#MyAppName}"; Flags: nowait postinstall skipifsilent

[Code]
procedure MigrateLegacyAutoStart;
var
  LegacyCommand: String;
  ExpectedLegacyCommand: String;
  NewCommand: String;
begin
  ExpectedLegacyCommand := '"' + ExpandConstant('{app}\VibeTimer.exe') + '" --background';
  if RegQueryStringValue(
    HKCU,
    'Software\Microsoft\Windows\CurrentVersion\Run',
    'VibeTimer',
    LegacyCommand
  ) and (CompareText(LegacyCommand, ExpectedLegacyCommand) = 0) then begin
    NewCommand := '"' + ExpandConstant('{app}\Vibemacro.exe') + '" --background';
    RegWriteStringValue(
      HKCU,
      'Software\Microsoft\Windows\CurrentVersion\Run',
      'Vibemacro',
      NewCommand
    );
    RegDeleteValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Run', 'VibeTimer');
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    MigrateLegacyAutoStart;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  AutoStartCommand: String;
  ExpectedCommand: String;
  LegacyCommand: String;
  ExpectedLegacyCommand: String;
begin
  if CurUninstallStep = usUninstall then begin
    ExpectedCommand := '"' + ExpandConstant('{app}\Vibemacro.exe') + '" --background';
    if RegQueryStringValue(
      HKCU,
      'Software\Microsoft\Windows\CurrentVersion\Run',
      'Vibemacro',
      AutoStartCommand
    ) and (CompareText(AutoStartCommand, ExpectedCommand) = 0) then
      RegDeleteValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Run', 'Vibemacro');

    ExpectedLegacyCommand := '"' + ExpandConstant('{app}\VibeTimer.exe') + '" --background';
    if RegQueryStringValue(
      HKCU,
      'Software\Microsoft\Windows\CurrentVersion\Run',
      'VibeTimer',
      LegacyCommand
    ) and (CompareText(LegacyCommand, ExpectedLegacyCommand) = 0) then
      RegDeleteValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Run', 'VibeTimer');
  end;
end;
