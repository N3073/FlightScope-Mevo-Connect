; Inno Setup script for FlightScope Mevo Connect.
; Build with:  ISCC.exe /DAppVersion=x.y.z installer\setup.iss   (after `cargo build --release`)
;
; Silent updates run this installer with /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /RELAUNCH.
; The previous network name and task choices are reused, and /RELAUNCH starts the app again.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#define AppName "FlightScope Mevo Connect"
#define AppExe "FlightScope-connect.exe"
#define AppUrl "https://github.com/N3073/FlightScope-Mevo-Connect"

[Setup]
AppId={{8F3C2A51-6B7E-4D2F-9A41-3E5C7B9D0F12}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=N3073
AppPublisherURL={#AppUrl}
AppUpdatesURL={#AppUrl}/releases
; Per-user install (%LOCALAPPDATA%\Programs) so the app can update itself without admin rights.
PrivilegesRequired=lowest
DefaultDirName={autopf}\{#AppName}
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
SetupIconFile=..\assets\icon.ico
ShowLanguageDialog=auto
ChangesEnvironment=yes
CloseApplications=yes
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
OutputDir=..\dist
OutputBaseFilename=FlightScope-Mevo-Connect-{#AppVersion}-Setup
Compression=lzma2
SolidCompression=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "finnish"; MessagesFile: "compiler:Languages\Finnish.isl"
Name: "swedish"; MessagesFile: "compiler:Languages\Swedish.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "italian"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "portuguese"; MessagesFile: "compiler:Languages\Portuguese.isl"

[CustomMessages]
english.SsidPageCaption=Flight monitor
english.SsidPageDescription=Which FlightScope Mevo should this PC connect to?
english.SsidPageText=Enter the name of the Mevo's wifi network, for example FS M2-123456.%n%nWindows must have connected to this network once before, so that its password is saved. If it hasn't, connect to it from the Windows wifi menu first.
english.SsidLabel=Wifi network name (SSID):
english.SsidRequired=Please enter the Mevo's wifi network name.
english.SsidNotMevo="%1" doesn't look like a FlightScope Mevo network name. Use it anyway?
english.StartupGroup=Startup:
english.StartupTask=Start automatically when I sign in to Windows

finnish.SsidPageCaption=Lentoratamittari
finnish.SsidPageDescription=Mihin FlightScope Mevoon tämä tietokone yhdistää?
finnish.SsidPageText=Kirjoita Mevon wifi-verkon nimi, esimerkiksi FS M2-123456.%n%nWindowsin on täytynyt yhdistää tähän verkkoon kerran aiemmin, jotta sen salasana on tallennettu. Jos näin ei ole, yhdistä siihen ensin Windowsin wifi-valikosta.
finnish.SsidLabel=Wifi-verkon nimi (SSID):
finnish.SsidRequired=Kirjoita Mevon wifi-verkon nimi.
finnish.SsidNotMevo="%1" ei näytä FlightScope Mevon verkon nimeltä. Käytetäänkö sitä silti?
finnish.StartupGroup=Käynnistys:
finnish.StartupTask=Käynnistä automaattisesti, kun kirjaudun Windowsiin

swedish.SsidPageCaption=Bollflygsmätare
swedish.SsidPageDescription=Vilken FlightScope Mevo ska den här datorn ansluta till?
swedish.SsidPageText=Ange namnet på Mevons wifi-nätverk, till exempel FS M2-123456.%n%nWindows måste ha anslutit till det här nätverket en gång tidigare, så att lösenordet är sparat. Om inte, anslut till det via Windows wifi-meny först.
swedish.SsidLabel=Wifi-nätverkets namn (SSID):
swedish.SsidRequired=Ange namnet på Mevons wifi-nätverk.
swedish.SsidNotMevo="%1" ser inte ut som namnet på ett FlightScope Mevo-nätverk. Använda det ändå?
swedish.StartupGroup=Uppstart:
swedish.StartupTask=Starta automatiskt när jag loggar in i Windows

german.SsidPageCaption=Launch Monitor
german.SsidPageDescription=Mit welchem FlightScope Mevo soll sich dieser PC verbinden?
german.SsidPageText=Geben Sie den Namen des Mevo-WLANs ein, zum Beispiel FS M2-123456.%n%nWindows muss sich bereits einmal mit diesem Netzwerk verbunden haben, damit das Passwort gespeichert ist. Falls nicht, verbinden Sie sich zuerst über das WLAN-Menü von Windows damit.
german.SsidLabel=WLAN-Name (SSID):
german.SsidRequired=Bitte geben Sie den Namen des Mevo-WLANs ein.
german.SsidNotMevo="%1" sieht nicht wie ein FlightScope-Mevo-Netzwerkname aus. Trotzdem verwenden?
german.StartupGroup=Autostart:
german.StartupTask=Bei der Windows-Anmeldung automatisch starten

italian.SsidPageCaption=Launch monitor
italian.SsidPageDescription=A quale FlightScope Mevo deve connettersi questo PC?
italian.SsidPageText=Inserisci il nome della rete Wi-Fi del Mevo, ad esempio FS M2-123456.%n%nWindows deve essersi già connesso una volta a questa rete, in modo che la password sia salvata. In caso contrario, connettiti prima dal menu Wi-Fi di Windows.
italian.SsidLabel=Nome della rete Wi-Fi (SSID):
italian.SsidRequired=Inserisci il nome della rete Wi-Fi del Mevo.
italian.SsidNotMevo="%1" non sembra il nome di una rete FlightScope Mevo. Usarlo comunque?
italian.StartupGroup=Avvio:
italian.StartupTask=Avvia automaticamente all'accesso a Windows

french.SsidPageCaption=Launch monitor
french.SsidPageDescription=À quel FlightScope Mevo ce PC doit-il se connecter ?
french.SsidPageText=Saisissez le nom du réseau Wi-Fi du Mevo, par exemple FS M2-123456.%n%nWindows doit s'être déjà connecté une fois à ce réseau pour que son mot de passe soit enregistré. Sinon, connectez-vous-y d'abord depuis le menu Wi-Fi de Windows.
french.SsidLabel=Nom du réseau Wi-Fi (SSID) :
french.SsidRequired=Veuillez saisir le nom du réseau Wi-Fi du Mevo.
french.SsidNotMevo=« %1 » ne ressemble pas à un nom de réseau FlightScope Mevo. L'utiliser quand même ?
french.StartupGroup=Démarrage :
french.StartupTask=Démarrer automatiquement à l'ouverture de session Windows

spanish.SsidPageCaption=Launch monitor
spanish.SsidPageDescription=¿A qué FlightScope Mevo debe conectarse este PC?
spanish.SsidPageText=Introduce el nombre de la red Wi-Fi del Mevo, por ejemplo FS M2-123456.%n%nWindows debe haberse conectado a esta red al menos una vez para que su contraseña esté guardada. Si no es así, conéctate primero desde el menú Wi-Fi de Windows.
spanish.SsidLabel=Nombre de la red Wi-Fi (SSID):
spanish.SsidRequired=Introduce el nombre de la red Wi-Fi del Mevo.
spanish.SsidNotMevo="%1" no parece un nombre de red de FlightScope Mevo. ¿Usarlo de todos modos?
spanish.StartupGroup=Inicio:
spanish.StartupTask=Iniciar automáticamente al iniciar sesión en Windows

portuguese.SsidPageCaption=Launch monitor
portuguese.SsidPageDescription=A que FlightScope Mevo deve este PC conectar-se?
portuguese.SsidPageText=Introduza o nome da rede Wi-Fi do Mevo, por exemplo FS M2-123456.%n%nO Windows tem de se ter conectado a esta rede pelo menos uma vez, para que a palavra-passe esteja guardada. Caso contrário, conecte-se primeiro a partir do menu Wi-Fi do Windows.
portuguese.SsidLabel=Nome da rede Wi-Fi (SSID):
portuguese.SsidRequired=Introduza o nome da rede Wi-Fi do Mevo.
portuguese.SsidNotMevo="%1" não parece um nome de rede FlightScope Mevo. Usar mesmo assim?
portuguese.StartupGroup=Arranque:
portuguese.StartupTask=Iniciar automaticamente ao iniciar sessão no Windows

[Tasks]
Name: "startup"; Description: "{cm:StartupTask}"; GroupDescription: "{cm:StartupGroup}"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon
Name: "{userstartup}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: startup

[InstallDelete]
; Unticking a task on reinstall doesn't remove its shortcut by itself.
Type: files; Name: "{userstartup}\{#AppName}.lnk"; Tasks: not startup
Type: files; Name: "{autodesktop}\{#AppName}.lnk"; Tasks: not desktopicon

[Registry]
; The app reads the network name from the user's FS_name environment variable.
Root: HKCU; Subkey: "Environment"; ValueType: string; ValueName: "FS_name"; ValueData: "{code:GetSsid}"; Flags: uninsdeletevalue; Check: SsidEntered

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
Filename: "{app}\{#AppExe}"; Flags: nowait; Check: IsRelaunch

[Code]
var
  SsidPage: TInputQueryWizardPage;

// The current network name: the user's FS_name variable, or else the system-wide one.
function ExistingSsid(): String;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'FS_name', Result) then
    if not RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'FS_name', Result) then
      Result := '';
end;

procedure InitializeWizard;
begin
  SsidPage := CreateInputQueryPage(wpWelcome,
    CustomMessage('SsidPageCaption'),
    CustomMessage('SsidPageDescription'),
    CustomMessage('SsidPageText'));
  SsidPage.Add(CustomMessage('SsidLabel'), False);
  SsidPage.Values[0] := ExistingSsid();
end;

function LooksLikeMevo(Ssid: String): Boolean;
begin
  Ssid := Lowercase(Ssid);
  Result := (Pos('fs', Ssid) = 1) or (Pos('mevo', Ssid) > 0);
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  Ssid: String;
begin
  Result := True;
  if CurPageID = SsidPage.ID then
  begin
    Ssid := Trim(SsidPage.Values[0]);
    if Ssid = '' then
    begin
      MsgBox(CustomMessage('SsidRequired'), mbError, MB_OK);
      Result := False;
    end
    else if not LooksLikeMevo(Ssid) then
      Result := MsgBox(FmtMessage(CustomMessage('SsidNotMevo'), [Ssid]), mbConfirmation, MB_YESNO) = IDYES;
  end;
end;

function GetSsid(Param: String): String;
begin
  Result := Trim(SsidPage.Values[0]);
end;

function SsidEntered(): Boolean;
begin
  Result := GetSsid('') <> '';
end;

function IsRelaunch(): Boolean;
var
  I: Integer;
begin
  Result := False;
  for I := 1 to ParamCount do
    if CompareText(ParamStr(I), '/RELAUNCH') = 0 then
      Result := True;
end;
