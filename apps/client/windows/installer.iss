[Setup]
AppName=TrustRAG
AppVersion=1.0.0
AppPublisher=TrustRAG Team
DefaultDirName={autopf}\TrustRAG
DefaultGroupName=TrustRAG
OutputDir=..\..\
OutputBaseFilename=TrustRAG-Setup-Windows-x64
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\client.exe
WizardStyle=modern

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\build\windows\x64\runner\Release\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\TrustRAG"; Filename: "{app}\client.exe"
Name: "{group}\{cm:UninstallProgram,TrustRAG}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\TrustRAG"; Filename: "{app}\client.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\client.exe"; Description: "{cm:LaunchProgram,TrustRAG}"; Flags: nowait postinstall skipifsilent
