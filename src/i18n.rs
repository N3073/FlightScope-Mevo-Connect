/// UI languages. The code is what gets saved in the settings.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Finnish,
    Swedish,
    German,
    Italian,
    French,
    Spanish,
    Portuguese,
}

impl Language {
    pub const ALL: [Language; 8] = [
        Language::English,
        Language::Finnish,
        Language::Swedish,
        Language::German,
        Language::Italian,
        Language::French,
        Language::Spanish,
        Language::Portuguese,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en-GB",
            Language::Finnish => "fi",
            Language::Swedish => "sv",
            Language::German => "de",
            Language::Italian => "it",
            Language::French => "fr",
            Language::Spanish => "es",
            Language::Portuguese => "pt",
        }
    }

    /// The language's name in that language, as shown in the selector.
    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English (UK)",
            Language::Finnish => "Suomi",
            Language::Swedish => "Svenska",
            Language::German => "Deutsch",
            Language::Italian => "Italiano",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::Portuguese => "Português",
        }
    }

    /// Matches a saved code or a system locale such as "fi-FI" by its language part.
    pub fn from_code(code: &str) -> Option<Language> {
        let prefix = code.split(['-', '_']).next()?.to_lowercase();
        Language::ALL.into_iter().find(|lang| lang.code().starts_with(&prefix))
    }

    /// The Windows display language if it is supported, otherwise English.
    pub fn from_system() -> Language {
        sys_locale::get_locale()
            .and_then(|locale| Language::from_code(&locale))
            .unwrap_or(Language::English)
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Language::English => &EN,
            Language::Finnish => &FI,
            Language::Swedish => &SV,
            Language::German => &DE,
            Language::Italian => &IT,
            Language::French => &FR,
            Language::Spanish => &ES,
            Language::Portuguese => &PT,
        }
    }
}

pub struct Strings {
    pub flight_monitor: &'static str,
    pub mevo_network: &'static str,
    pub mevo_network_hover: &'static str,
    pub not_mevo: &'static str,
    pub not_mevo_hover: &'static str,
    pub no_profile: &'static str,
    pub not_connected: &'static str,
    /// `{n}` is replaced with the attempt number.
    pub connecting: &'static str,
    pub connected: &'static str,
    pub retry_hint: &'static str,
    pub closing: &'static str,
    pub connect: &'static str,
    pub cancel: &'static str,
    pub fs_name_missing: &'static str,
    pub fs_name_missing_hint: &'static str,
    pub settings: &'static str,
    pub language: &'static str,
    pub back: &'static str,
}

const EN: Strings = Strings {
    flight_monitor: "Flight monitor",
    mevo_network: "✔ Mevo network",
    mevo_network_hover: "The network name matches the FlightScope Mevo naming pattern.",
    not_mevo: "⚠ Not a Mevo name",
    not_mevo_hover: "The network name does not match the FlightScope Mevo naming pattern. Check FS_name.",
    no_profile: "Windows has no saved profile for this network. Connect to it once from the Windows wifi menu first.",
    not_connected: "Not connected",
    connecting: "Connecting… (attempt {n})",
    connected: "Connected",
    retry_hint: "Make sure the Mevo is switched on. Retrying until it connects.",
    closing: "Closing automatically…",
    connect: "Connect",
    cancel: "Cancel",
    fs_name_missing: "FS_name is not set",
    fs_name_missing_hint: "Set the FS_name environment variable to the name of the Mevo's wifi network and restart the program.",
    settings: "Settings",
    language: "Language",
    back: "Back",
};

const FI: Strings = Strings {
    flight_monitor: "Lentoratamittari",
    mevo_network: "✔ Mevo-verkko",
    mevo_network_hover: "Verkon nimi vastaa FlightScope Mevon nimeämiskaavaa.",
    not_mevo: "⚠ Ei Mevo-nimi",
    not_mevo_hover: "Verkon nimi ei vastaa FlightScope Mevon nimeämiskaavaa. Tarkista FS_name.",
    no_profile: "Windowsissa ei ole tallennettua profiilia tälle verkolle. Yhdistä siihen ensin kerran Windowsin wifi-valikosta.",
    not_connected: "Ei yhdistetty",
    connecting: "Yhdistetään… (yritys {n})",
    connected: "Yhdistetty",
    retry_hint: "Varmista, että Mevo on päällä. Yritetään uudelleen, kunnes yhteys muodostuu.",
    closing: "Suljetaan automaattisesti…",
    connect: "Yhdistä",
    cancel: "Peruuta",
    fs_name_missing: "FS_name-muuttujaa ei ole asetettu",
    fs_name_missing_hint: "Aseta ympäristömuuttujan FS_name arvoksi Mevon wifi-verkon nimi ja käynnistä ohjelma uudelleen.",
    settings: "Asetukset",
    language: "Kieli",
    back: "Takaisin",
};

const SV: Strings = Strings {
    flight_monitor: "Bollflygsmätare",
    mevo_network: "✔ Mevo-nätverk",
    mevo_network_hover: "Nätverksnamnet matchar FlightScope Mevos namnmönster.",
    not_mevo: "⚠ Inte ett Mevo-namn",
    not_mevo_hover: "Nätverksnamnet matchar inte FlightScope Mevos namnmönster. Kontrollera FS_name.",
    no_profile: "Windows har ingen sparad profil för det här nätverket. Anslut till det en gång via Windows wifi-meny först.",
    not_connected: "Inte ansluten",
    connecting: "Ansluter… (försök {n})",
    connected: "Ansluten",
    retry_hint: "Se till att Mevo är påslagen. Försöker igen tills anslutningen lyckas.",
    closing: "Stängs automatiskt…",
    connect: "Anslut",
    cancel: "Avbryt",
    fs_name_missing: "FS_name är inte angiven",
    fs_name_missing_hint: "Ange miljövariabeln FS_name till namnet på Mevons wifi-nätverk och starta om programmet.",
    settings: "Inställningar",
    language: "Språk",
    back: "Tillbaka",
};

const DE: Strings = Strings {
    flight_monitor: "Launch Monitor",
    mevo_network: "✔ Mevo-Netzwerk",
    mevo_network_hover: "Der Netzwerkname entspricht dem Namensschema von FlightScope Mevo.",
    not_mevo: "⚠ Kein Mevo-Name",
    not_mevo_hover: "Der Netzwerkname entspricht nicht dem Namensschema von FlightScope Mevo. Bitte FS_name prüfen.",
    no_profile: "Windows hat kein gespeichertes Profil für dieses Netzwerk. Verbinden Sie sich zuerst einmal über das WLAN-Menü von Windows damit.",
    not_connected: "Nicht verbunden",
    connecting: "Verbindung wird hergestellt… (Versuch {n})",
    connected: "Verbunden",
    retry_hint: "Stellen Sie sicher, dass der Mevo eingeschaltet ist. Es wird erneut versucht, bis die Verbindung steht.",
    closing: "Wird automatisch geschlossen…",
    connect: "Verbinden",
    cancel: "Abbrechen",
    fs_name_missing: "FS_name ist nicht gesetzt",
    fs_name_missing_hint: "Setzen Sie die Umgebungsvariable FS_name auf den Namen des Mevo-WLANs und starten Sie das Programm neu.",
    settings: "Einstellungen",
    language: "Sprache",
    back: "Zurück",
};

const IT: Strings = Strings {
    flight_monitor: "Launch monitor",
    mevo_network: "✔ Rete Mevo",
    mevo_network_hover: "Il nome della rete corrisponde allo schema di denominazione di FlightScope Mevo.",
    not_mevo: "⚠ Nome non Mevo",
    not_mevo_hover: "Il nome della rete non corrisponde allo schema di denominazione di FlightScope Mevo. Controlla FS_name.",
    no_profile: "Windows non ha un profilo salvato per questa rete. Connettiti prima una volta dal menu Wi-Fi di Windows.",
    not_connected: "Non connesso",
    connecting: "Connessione in corso… (tentativo {n})",
    connected: "Connesso",
    retry_hint: "Assicurati che il Mevo sia acceso. Nuovi tentativi fino alla connessione.",
    closing: "Chiusura automatica…",
    connect: "Connetti",
    cancel: "Annulla",
    fs_name_missing: "FS_name non è impostata",
    fs_name_missing_hint: "Imposta la variabile d'ambiente FS_name sul nome della rete Wi-Fi del Mevo e riavvia il programma.",
    settings: "Impostazioni",
    language: "Lingua",
    back: "Indietro",
};

const FR: Strings = Strings {
    flight_monitor: "Launch monitor",
    mevo_network: "✔ Réseau Mevo",
    mevo_network_hover: "Le nom du réseau correspond au format de nommage FlightScope Mevo.",
    not_mevo: "⚠ Nom non Mevo",
    not_mevo_hover: "Le nom du réseau ne correspond pas au format de nommage FlightScope Mevo. Vérifiez FS_name.",
    no_profile: "Windows n'a aucun profil enregistré pour ce réseau. Connectez-vous-y d'abord une fois depuis le menu Wi-Fi de Windows.",
    not_connected: "Non connecté",
    connecting: "Connexion… (tentative {n})",
    connected: "Connecté",
    retry_hint: "Assurez-vous que le Mevo est allumé. Nouvelle tentative jusqu'à la connexion.",
    closing: "Fermeture automatique…",
    connect: "Connecter",
    cancel: "Annuler",
    fs_name_missing: "FS_name n'est pas défini",
    fs_name_missing_hint: "Définissez la variable d'environnement FS_name avec le nom du réseau Wi-Fi du Mevo, puis redémarrez le programme.",
    settings: "Paramètres",
    language: "Langue",
    back: "Retour",
};

const ES: Strings = Strings {
    flight_monitor: "Launch monitor",
    mevo_network: "✔ Red Mevo",
    mevo_network_hover: "El nombre de la red coincide con el patrón de nombres de FlightScope Mevo.",
    not_mevo: "⚠ No es un nombre Mevo",
    not_mevo_hover: "El nombre de la red no coincide con el patrón de nombres de FlightScope Mevo. Comprueba FS_name.",
    no_profile: "Windows no tiene un perfil guardado para esta red. Conéctate a ella una vez desde el menú Wi-Fi de Windows primero.",
    not_connected: "No conectado",
    connecting: "Conectando… (intento {n})",
    connected: "Conectado",
    retry_hint: "Asegúrate de que el Mevo esté encendido. Se reintentará hasta que se conecte.",
    closing: "Cerrando automáticamente…",
    connect: "Conectar",
    cancel: "Cancelar",
    fs_name_missing: "FS_name no está definida",
    fs_name_missing_hint: "Define la variable de entorno FS_name con el nombre de la red Wi-Fi del Mevo y reinicia el programa.",
    settings: "Ajustes",
    language: "Idioma",
    back: "Volver",
};

const PT: Strings = Strings {
    flight_monitor: "Launch monitor",
    mevo_network: "✔ Rede Mevo",
    mevo_network_hover: "O nome da rede corresponde ao padrão de nomes do FlightScope Mevo.",
    not_mevo: "⚠ Não é um nome Mevo",
    not_mevo_hover: "O nome da rede não corresponde ao padrão de nomes do FlightScope Mevo. Verifique FS_name.",
    no_profile: "O Windows não tem um perfil guardado para esta rede. Conecte-se a ela uma vez a partir do menu Wi-Fi do Windows primeiro.",
    not_connected: "Não conectado",
    connecting: "A conectar… (tentativa {n})",
    connected: "Conectado",
    retry_hint: "Certifique-se de que o Mevo está ligado. Nova tentativa até conseguir a conexão.",
    closing: "A fechar automaticamente…",
    connect: "Conectar",
    cancel: "Cancelar",
    fs_name_missing: "FS_name não está definida",
    fs_name_missing_hint: "Defina a variável de ambiente FS_name com o nome da rede Wi-Fi do Mevo e reinicie o programa.",
    settings: "Definições",
    language: "Idioma",
    back: "Voltar",
};
