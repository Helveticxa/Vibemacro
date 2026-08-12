#ifndef MyAppId
  #define MyAppId "{{22C5EF59-EEA3-4F81-BA7B-C3EF16383980}"
#endif
#ifndef MyOutputDir
  #define MyOutputDir "..\qa\installer-upgrade\fixtures"
#endif

[Setup]
AppId={#MyAppId}
AppName=VibeTimer
AppVersion=1.0.0
AppVerName=VibeTimer 1.0.0
DefaultDirName={localappdata}\Programs\VibeTimer-QA
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#MyOutputDir}
OutputBaseFilename=VibeTimer-Setup-1.0.0-QA
Uninstallable=yes
CloseApplications=yes
RestartApplications=no
Compression=lzma2
SolidCompression=yes

[Files]
Source: "..\target\release\Vibemacro.exe"; DestDir: "{app}"; DestName: "VibeTimer.exe"; Flags: ignoreversion
