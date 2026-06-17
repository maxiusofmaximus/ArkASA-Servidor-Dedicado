Unicode True
SetCompressor /SOLID lzma

!addplugindir /x86-unicode "plugins\x86-unicode"
!addplugindir /amd64-unicode "plugins\amd64-unicode"

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "WinVer.nsh"
!include "Sections.nsh"
!include "WinMessages.nsh"

Name "ARK ASA Configuration Manager"
OutFile "ARK-ASA-Full-Setup.exe"
InstallDir "C:\Program Files\ARK ASA Config Manager"
InstallDirRegKey HKLM "Software\ARKASAConfigManager" "InstallDir"
RequestExecutionLevel admin

VIProductVersion "1.4.0.0"
VIAddVersionKey "ProductName"     "ARK ASA Configuration Manager Setup"
VIAddVersionKey "ProductVersion"  "1.4.0"
VIAddVersionKey "FileDescription" "Full installer for ARK ASA Config Manager"
VIAddVersionKey "LegalCopyright"  "MIT License - 2026"
VIAddVersionKey "CompanyName"     "ARK ASA Config Manager"

!define MUI_ICON   "..\src-tauri\icons\icon.ico"
!define MUI_UNICON "..\src-tauri\icons\icon.ico"

!define MUI_ABORTWARNING

!define MUI_WELCOMEPAGE_TITLE        "$(TITLE_WELCOME)"
!define MUI_WELCOMEPAGE_TEXT         "$(TEXT_WELCOME)"
!define MUI_FINISHPAGE_TITLE         "$(TITLE_FINISH)"
!define MUI_FINISHPAGE_TEXT          "$(TEXT_FINISH)"
!define MUI_FINISHPAGE_RUN           "$INSTDIR\ark-asa-config.exe"
!define MUI_FINISHPAGE_RUN_TEXT      "$(TEXT_FINISH_RUN)"
!define MUI_FINISHPAGE_LINK          "$(TEXT_FINISH_LINK)"
!define MUI_FINISHPAGE_LINK_LOCATION "https://maxiusofmaximus.github.io/ArkASA-Servidor-Dedicado/"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "license.txt"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
Page custom ShortcutPage ShortcutPageLeave
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; English first = default when no preference saved
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "German"
!insertmacro MUI_LANGUAGE "Italian"
!insertmacro MUI_LANGUAGE "PortugueseBR"
!insertmacro MUI_LANGUAGE "Russian"
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "Japanese"
!insertmacro MUI_LANGUAGE "Korean"

; ─── TITLE_WELCOME ───────────────────────────────────────────────────────────
LangString TITLE_WELCOME ${LANG_ENGLISH}      "Welcome to ARK ASA Config Manager"
LangString TITLE_WELCOME ${LANG_SPANISH}      "Bienvenido a ARK ASA Config Manager"
LangString TITLE_WELCOME ${LANG_FRENCH}       "Bienvenue dans ARK ASA Config Manager"
LangString TITLE_WELCOME ${LANG_GERMAN}       "Willkommen bei ARK ASA Config Manager"
LangString TITLE_WELCOME ${LANG_ITALIAN}      "Benvenuto in ARK ASA Config Manager"
LangString TITLE_WELCOME ${LANG_PORTUGUESEBR} "Bem-vindo ao ARK ASA Config Manager"
LangString TITLE_WELCOME ${LANG_RUSSIAN}      "Welcome to ARK ASA Config Manager"
LangString TITLE_WELCOME ${LANG_SIMPCHINESE}  "欢迎使用 ARK ASA Config Manager"
LangString TITLE_WELCOME ${LANG_JAPANESE}     "ARK ASA Config Manager へようこそ"
LangString TITLE_WELCOME ${LANG_KOREAN}       "ARK ASA Config Manager에 오신 것을 환영합니다"

; ─── TEXT_WELCOME ─────────────────────────────────────────────────────────────
LangString TEXT_WELCOME ${LANG_ENGLISH}      "This wizard will install everything needed to run an ARK: Survival Ascended dedicated server:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (Valve tool)$\r$\n  - ARK: Survival Ascended Server (~100 GB)$\r$\n$\r$\nNOTE: The server download requires approx.$\r$\n100 GB of free space and 1-2 hours.$\r$\n$\r$\nClick Next to continue."
LangString TEXT_WELCOME ${LANG_SPANISH}      "Este asistente instalara todo lo necesario para tu servidor ARK: Survival Ascended:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (herramienta de Valve)$\r$\n  - Servidor ARK: Survival Ascended (~100 GB)$\r$\n$\r$\nNOTA: La descarga del servidor requiere aprox.$\r$\n100 GB de espacio libre y 1-2 horas.$\r$\n$\r$\nHaz clic en Siguiente para continuar."
LangString TEXT_WELCOME ${LANG_FRENCH}       "Cet assistant installera tout le necessaire pour un serveur ARK: Survival Ascended:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (outil Valve)$\r$\n  - Serveur ARK: Survival Ascended (~100 Go)$\r$\n$\r$\nREMARQUE: Le telechargement requiert environ$\r$\n100 Go et 1-2 heures.$\r$\n$\r$\nCliquez sur Suivant pour continuer."
LangString TEXT_WELCOME ${LANG_GERMAN}       "Dieser Assistent installiert alles fuer einen ARK: Survival Ascended-Server:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (Valve-Tool)$\r$\n  - ARK: Survival Ascended Server (~100 GB)$\r$\n$\r$\nHINWEIS: Der Download benoetigt ca.$\r$\n100 GB und 1-2 Stunden.$\r$\n$\r$\nKlicken Sie auf Weiter."
LangString TEXT_WELCOME ${LANG_ITALIAN}      "Questa procedura installera tutto il necessario per un server ARK: Survival Ascended:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (strumento Valve)$\r$\n  - Server ARK: Survival Ascended (~100 GB)$\r$\n$\r$\nNOTA: Il download richiede circa$\r$\n100 GB e 1-2 ore.$\r$\n$\r$\nFare clic su Avanti per continuare."
LangString TEXT_WELCOME ${LANG_PORTUGUESEBR} "Este assistente instalara tudo necessario para um servidor ARK: Survival Ascended:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (ferramenta da Valve)$\r$\n  - Servidor ARK: Survival Ascended (~100 GB)$\r$\n$\r$\nNOTA: O download requer aprox.$\r$\n100 GB e 1-2 horas.$\r$\n$\r$\nClique em Avancar para continuar."
LangString TEXT_WELCOME ${LANG_RUSSIAN}      "This wizard will install everything needed for an ARK: Survival Ascended server:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (Valve tool)$\r$\n  - ARK: Survival Ascended Server (~100 GB)$\r$\n$\r$\nNOTE: Server download requires approx.$\r$\n100 GB and 1-2 hours.$\r$\n$\r$\nClick Next to continue."
LangString TEXT_WELCOME ${LANG_SIMPCHINESE}  "此向导将安装运行ARK:方舟生存进化专用服务器所需的一切:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (Valve工具)$\r$\n  - ARK服务器 (~100 GB)$\r$\n$\r$\n注意: 服务器下载约需100 GB空间和1-2小时。$\r$\n$\r$\n点击下一步继续。"
LangString TEXT_WELCOME ${LANG_JAPANESE}     "このウィザードはARK: Survival Ascended専用サーバーに必要なものをすべてインストールします:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (Valveツール)$\r$\n  - ARKサーバー (~100 GB)$\r$\n$\r$\n注意: 約100GBのspace と1-2時間が必要です。$\r$\n$\r$\n次へをクリックして続けてください。"
LangString TEXT_WELCOME ${LANG_KOREAN}       "이 마법사는 ARK: Survival Ascended 전용 서버에 필요한 모든 것을 설치합니다:$\r$\n$\r$\n  - ARK ASA Configuration Manager$\r$\n  - Visual C++ Redistributable 2022$\r$\n  - SteamCMD (Valve 도구)$\r$\n  - ARK 서버 (~100 GB)$\r$\n$\r$\n참고: 약 100GB 공간과 1-2시간이 필요합니다.$\r$\n$\r$\n계속하려면 다음을 클릭하십시오."

; ─── TITLE_FINISH ─────────────────────────────────────────────────────────────
LangString TITLE_FINISH ${LANG_ENGLISH}      "Installation Complete!"
LangString TITLE_FINISH ${LANG_SPANISH}      "Instalacion completada!"
LangString TITLE_FINISH ${LANG_FRENCH}       "Installation terminee!"
LangString TITLE_FINISH ${LANG_GERMAN}       "Installation abgeschlossen!"
LangString TITLE_FINISH ${LANG_ITALIAN}      "Installazione completata!"
LangString TITLE_FINISH ${LANG_PORTUGUESEBR} "Instalacao concluida!"
LangString TITLE_FINISH ${LANG_RUSSIAN}      "Installation Complete!"
LangString TITLE_FINISH ${LANG_SIMPCHINESE}  "安装完成！"
LangString TITLE_FINISH ${LANG_JAPANESE}     "インストール完了！"
LangString TITLE_FINISH ${LANG_KOREAN}       "설치 완료!"

; ─── TEXT_FINISH ──────────────────────────────────────────────────────────────
LangString TEXT_FINISH ${LANG_ENGLISH}      "ARK ASA Configuration Manager and all components were installed successfully.$\r$\n$\r$\nYour ARK: Survival Ascended server is ready.$\r$\nOpen the app to configure and start it."
LangString TEXT_FINISH ${LANG_SPANISH}      "ARK ASA Configuration Manager y todos sus componentes se instalaron correctamente.$\r$\n$\r$\nTu servidor de ARK: Survival Ascended esta listo.$\r$\nAbre la aplicacion para configurarlo e iniciarlo."
LangString TEXT_FINISH ${LANG_FRENCH}       "ARK ASA Configuration Manager et tous ses composants ont ete installes.$\r$\n$\r$\nVotre serveur ARK: Survival Ascended est pret.$\r$\nOuvrez l'application pour le configurer et le demarrer."
LangString TEXT_FINISH ${LANG_GERMAN}       "ARK ASA Configuration Manager und alle Komponenten wurden erfolgreich installiert.$\r$\n$\r$\nIhr ARK-Server ist bereit.$\r$\nOeffnen Sie die App, um ihn zu konfigurieren und zu starten."
LangString TEXT_FINISH ${LANG_ITALIAN}      "ARK ASA Configuration Manager e tutti i componenti sono stati installati.$\r$\n$\r$\nIl tuo server ARK e pronto.$\r$\nApri l'app per configurarlo e avviarlo."
LangString TEXT_FINISH ${LANG_PORTUGUESEBR} "ARK ASA Configuration Manager e todos os componentes foram instalados.$\r$\n$\r$\nSeu servidor ARK esta pronto.$\r$\nAbra o aplicativo para configura-lo e inicia-lo."
LangString TEXT_FINISH ${LANG_RUSSIAN}      "ARK ASA Configuration Manager and all components installed successfully.$\r$\n$\r$\nYour ARK server is ready.$\r$\nOpen the app to configure and start it."
LangString TEXT_FINISH ${LANG_SIMPCHINESE}  "ARK ASA Configuration Manager及所有组件已成功安装。$\r$\n$\r$\n您的ARK服务器已就绪。$\r$\n打开应用程序进行配置并启动。"
LangString TEXT_FINISH ${LANG_JAPANESE}     "ARK ASA Configuration Managerとすべてのコンポーネントが正常にインストールされました。$\r$\n$\r$\nARKサーバーの準備ができました。$\r$\nアプリを開いて設定・起動してください。"
LangString TEXT_FINISH ${LANG_KOREAN}       "ARK ASA Configuration Manager와 모든 구성 요소가 성공적으로 설치되었습니다.$\r$\n$\r$\nARK 서버가 준비되었습니다.$\r$\n앱을 열어 구성하고 시작하세요."

; ─── TEXT_FINISH_RUN / TEXT_FINISH_LINK ───────────────────────────────────────
LangString TEXT_FINISH_RUN ${LANG_ENGLISH}      "Launch ARK ASA Configuration Manager"
LangString TEXT_FINISH_RUN ${LANG_SPANISH}      "Iniciar ARK ASA Configuration Manager"
LangString TEXT_FINISH_RUN ${LANG_FRENCH}       "Lancer ARK ASA Configuration Manager"
LangString TEXT_FINISH_RUN ${LANG_GERMAN}       "ARK ASA Configuration Manager starten"
LangString TEXT_FINISH_RUN ${LANG_ITALIAN}      "Avvia ARK ASA Configuration Manager"
LangString TEXT_FINISH_RUN ${LANG_PORTUGUESEBR} "Iniciar ARK ASA Configuration Manager"
LangString TEXT_FINISH_RUN ${LANG_RUSSIAN}      "Launch ARK ASA Configuration Manager"
LangString TEXT_FINISH_RUN ${LANG_SIMPCHINESE}  "启动 ARK ASA Configuration Manager"
LangString TEXT_FINISH_RUN ${LANG_JAPANESE}     "ARK ASA Configuration Manager を起動"
LangString TEXT_FINISH_RUN ${LANG_KOREAN}       "ARK ASA Configuration Manager 시작"

LangString TEXT_FINISH_LINK ${LANG_ENGLISH}      "View online documentation"
LangString TEXT_FINISH_LINK ${LANG_SPANISH}      "Ver documentacion online"
LangString TEXT_FINISH_LINK ${LANG_FRENCH}       "Voir la documentation en ligne"
LangString TEXT_FINISH_LINK ${LANG_GERMAN}       "Online-Dokumentation anzeigen"
LangString TEXT_FINISH_LINK ${LANG_ITALIAN}      "Visualizza documentazione online"
LangString TEXT_FINISH_LINK ${LANG_PORTUGUESEBR} "Ver documentacao online"
LangString TEXT_FINISH_LINK ${LANG_RUSSIAN}      "View online documentation"
LangString TEXT_FINISH_LINK ${LANG_SIMPCHINESE}  "查看在线文档"
LangString TEXT_FINISH_LINK ${LANG_JAPANESE}     "オンラインドキュメントを見る"
LangString TEXT_FINISH_LINK ${LANG_KOREAN}       "온라인 문서 보기"

; ─── SHORTCUT PAGE ────────────────────────────────────────────────────────────
LangString SHORTCUT_LABEL     ${LANG_ENGLISH}      "Select where to create shortcuts:"
LangString SHORTCUT_LABEL     ${LANG_SPANISH}      "Selecciona donde crear los accesos directos:"
LangString SHORTCUT_LABEL     ${LANG_FRENCH}       "Selectionnez ou creer des raccourcis:"
LangString SHORTCUT_LABEL     ${LANG_GERMAN}       "Waehlen Sie aus, wo Verknuepfungen erstellt werden:"
LangString SHORTCUT_LABEL     ${LANG_ITALIAN}      "Seleziona dove creare i collegamenti:"
LangString SHORTCUT_LABEL     ${LANG_PORTUGUESEBR} "Selecione onde criar atalhos:"
LangString SHORTCUT_LABEL     ${LANG_RUSSIAN}      "Select where to create shortcuts:"
LangString SHORTCUT_LABEL     ${LANG_SIMPCHINESE}  "选择创建快捷方式的位置："
LangString SHORTCUT_LABEL     ${LANG_JAPANESE}     "ショートカットを作成する場所を選択してください："
LangString SHORTCUT_LABEL     ${LANG_KOREAN}       "바로 가기를 만들 위치를 선택하세요："

LangString SHORTCUT_DESKTOP   ${LANG_ENGLISH}      "Create shortcut on the Desktop"
LangString SHORTCUT_DESKTOP   ${LANG_SPANISH}      "Crear acceso directo en el Escritorio"
LangString SHORTCUT_DESKTOP   ${LANG_FRENCH}       "Creer un raccourci sur le Bureau"
LangString SHORTCUT_DESKTOP   ${LANG_GERMAN}       "Verknuepfung auf dem Desktop erstellen"
LangString SHORTCUT_DESKTOP   ${LANG_ITALIAN}      "Crea collegamento sul Desktop"
LangString SHORTCUT_DESKTOP   ${LANG_PORTUGUESEBR} "Criar atalho na Area de Trabalho"
LangString SHORTCUT_DESKTOP   ${LANG_RUSSIAN}      "Create shortcut on the Desktop"
LangString SHORTCUT_DESKTOP   ${LANG_SIMPCHINESE}  "在桌面创建快捷方式"
LangString SHORTCUT_DESKTOP   ${LANG_JAPANESE}     "デスクトップにショートカットを作成"
LangString SHORTCUT_DESKTOP   ${LANG_KOREAN}       "바탕 화면에 바로 가기 만들기"

LangString SHORTCUT_STARTMENU ${LANG_ENGLISH}      "Create shortcut in the Start Menu"
LangString SHORTCUT_STARTMENU ${LANG_SPANISH}      "Crear acceso directo en el Menu Inicio"
LangString SHORTCUT_STARTMENU ${LANG_FRENCH}       "Creer un raccourci dans le Menu Demarrer"
LangString SHORTCUT_STARTMENU ${LANG_GERMAN}       "Verknuepfung im Startmenue erstellen"
LangString SHORTCUT_STARTMENU ${LANG_ITALIAN}      "Crea collegamento nel Menu Start"
LangString SHORTCUT_STARTMENU ${LANG_PORTUGUESEBR} "Criar atalho no Menu Iniciar"
LangString SHORTCUT_STARTMENU ${LANG_RUSSIAN}      "Create shortcut in the Start Menu"
LangString SHORTCUT_STARTMENU ${LANG_SIMPCHINESE}  "在开始菜单创建快捷方式"
LangString SHORTCUT_STARTMENU ${LANG_JAPANESE}     "スタートメニューにショートカットを作成"
LangString SHORTCUT_STARTMENU ${LANG_KOREAN}       "시작 메뉴에 바로 가기 만들기"

; ─── SECTION NAMES ────────────────────────────────────────────────────────────
LangString SEC_APP_NAME   ${LANG_ENGLISH}      "ARK ASA Config Manager (required)"
LangString SEC_APP_NAME   ${LANG_SPANISH}      "ARK ASA Config Manager (requerido)"
LangString SEC_APP_NAME   ${LANG_FRENCH}       "ARK ASA Config Manager (requis)"
LangString SEC_APP_NAME   ${LANG_GERMAN}       "ARK ASA Config Manager (erforderlich)"
LangString SEC_APP_NAME   ${LANG_ITALIAN}      "ARK ASA Config Manager (obbligatorio)"
LangString SEC_APP_NAME   ${LANG_PORTUGUESEBR} "ARK ASA Config Manager (obrigatorio)"
LangString SEC_APP_NAME   ${LANG_RUSSIAN}      "ARK ASA Config Manager (required)"
LangString SEC_APP_NAME   ${LANG_SIMPCHINESE}  "ARK ASA Config Manager（必需）"
LangString SEC_APP_NAME   ${LANG_JAPANESE}     "ARK ASA Config Manager（必須）"
LangString SEC_APP_NAME   ${LANG_KOREAN}       "ARK ASA Config Manager（필수）"

LangString SEC_VC_NAME    ${LANG_ENGLISH}      "Visual C++ Redistributable 2022 (recommended)"
LangString SEC_VC_NAME    ${LANG_SPANISH}      "Visual C++ Redistributable 2022 (recomendado)"
LangString SEC_VC_NAME    ${LANG_FRENCH}       "Visual C++ Redistributable 2022 (recommande)"
LangString SEC_VC_NAME    ${LANG_GERMAN}       "Visual C++ Redistributable 2022 (empfohlen)"
LangString SEC_VC_NAME    ${LANG_ITALIAN}      "Visual C++ Redistributable 2022 (consigliato)"
LangString SEC_VC_NAME    ${LANG_PORTUGUESEBR} "Visual C++ Redistributable 2022 (recomendado)"
LangString SEC_VC_NAME    ${LANG_RUSSIAN}      "Visual C++ Redistributable 2022 (recommended)"
LangString SEC_VC_NAME    ${LANG_SIMPCHINESE}  "Visual C++ Redistributable 2022（推荐）"
LangString SEC_VC_NAME    ${LANG_JAPANESE}     "Visual C++ Redistributable 2022（推奨）"
LangString SEC_VC_NAME    ${LANG_KOREAN}       "Visual C++ Redistributable 2022（권장）"

LangString SEC_STEAM_NAME ${LANG_ENGLISH}      "SteamCMD (required for server)"
LangString SEC_STEAM_NAME ${LANG_SPANISH}      "SteamCMD (necesario para el servidor)"
LangString SEC_STEAM_NAME ${LANG_FRENCH}       "SteamCMD (requis pour le serveur)"
LangString SEC_STEAM_NAME ${LANG_GERMAN}       "SteamCMD (fuer den Server erforderlich)"
LangString SEC_STEAM_NAME ${LANG_ITALIAN}      "SteamCMD (necessario per il server)"
LangString SEC_STEAM_NAME ${LANG_PORTUGUESEBR} "SteamCMD (necessario para o servidor)"
LangString SEC_STEAM_NAME ${LANG_RUSSIAN}      "SteamCMD (required for server)"
LangString SEC_STEAM_NAME ${LANG_SIMPCHINESE}  "SteamCMD（服务器必需）"
LangString SEC_STEAM_NAME ${LANG_JAPANESE}     "SteamCMD（サーバーに必要）"
LangString SEC_STEAM_NAME ${LANG_KOREAN}       "SteamCMD（서버에 필요）"

LangString SEC_ARK_NAME   ${LANG_ENGLISH}      "ARK: Survival Ascended Dedicated Server (~100 GB)"
LangString SEC_ARK_NAME   ${LANG_SPANISH}      "ARK: Survival Ascended - Servidor Dedicado (~100 GB)"
LangString SEC_ARK_NAME   ${LANG_FRENCH}       "ARK: Survival Ascended - Serveur dedie (~100 Go)"
LangString SEC_ARK_NAME   ${LANG_GERMAN}       "ARK: Survival Ascended - Dedicated Server (~100 GB)"
LangString SEC_ARK_NAME   ${LANG_ITALIAN}      "ARK: Survival Ascended - Server dedicato (~100 GB)"
LangString SEC_ARK_NAME   ${LANG_PORTUGUESEBR} "ARK: Survival Ascended - Servidor dedicado (~100 GB)"
LangString SEC_ARK_NAME   ${LANG_RUSSIAN}      "ARK: Survival Ascended Dedicated Server (~100 GB)"
LangString SEC_ARK_NAME   ${LANG_SIMPCHINESE}  "ARK:生存进化 专用服务器（~100 GB）"
LangString SEC_ARK_NAME   ${LANG_JAPANESE}     "ARK: Survival Ascended 専用サーバー（~100 GB）"
LangString SEC_ARK_NAME   ${LANG_KOREAN}       "ARK: Survival Ascended 전용 서버（~100 GB）"

; ─── DESCRIPTIONS ─────────────────────────────────────────────────────────────
LangString DESC_APP   ${LANG_ENGLISH}      "Desktop app to manage and launch your ARK server. Required."
LangString DESC_APP   ${LANG_SPANISH}      "App de escritorio para gestionar y lanzar tu servidor ARK. Obligatorio."
LangString DESC_APP   ${LANG_FRENCH}       "Application pour gerer et lancer votre serveur ARK. Obligatoire."
LangString DESC_APP   ${LANG_GERMAN}       "Desktop-App zum Verwalten und Starten des ARK-Servers. Erforderlich."
LangString DESC_APP   ${LANG_ITALIAN}      "App desktop per gestire e avviare il server ARK. Obbligatoria."
LangString DESC_APP   ${LANG_PORTUGUESEBR} "App para gerenciar e iniciar seu servidor ARK. Obrigatorio."
LangString DESC_APP   ${LANG_RUSSIAN}      "Desktop app to manage and launch your ARK server. Required."
LangString DESC_APP   ${LANG_SIMPCHINESE}  "用于管理和启动ARK服务器的桌面应用。必需。"
LangString DESC_APP   ${LANG_JAPANESE}     "ARKサーバーを管理・起動するデスクトップアプリ。必須。"
LangString DESC_APP   ${LANG_KOREAN}       "ARK 서버를 관리하고 시작하는 데스크톱 앱. 필수."

LangString DESC_VC    ${LANG_ENGLISH}      "Microsoft C++ runtime required by ARK server. Recommended on first install."
LangString DESC_VC    ${LANG_SPANISH}      "Libreria C++ de Microsoft necesaria para el servidor ARK. Recomendado en primera instalacion."
LangString DESC_VC    ${LANG_FRENCH}       "Runtime C++ Microsoft requis par le serveur ARK. Recommande pour la premiere installation."
LangString DESC_VC    ${LANG_GERMAN}       "Microsoft C++ Runtime fuer ARK-Server. Bei Erstinstallation empfohlen."
LangString DESC_VC    ${LANG_ITALIAN}      "Runtime C++ Microsoft richiesto dal server ARK. Consigliato alla prima installazione."
LangString DESC_VC    ${LANG_PORTUGUESEBR} "Runtime C++ da Microsoft necessario para o servidor ARK. Recomendado na primeira instalacao."
LangString DESC_VC    ${LANG_RUSSIAN}      "Microsoft C++ runtime required by ARK server. Recommended on first install."
LangString DESC_VC    ${LANG_SIMPCHINESE}  "ARK服务器所需的微软C++运行库。首次安装推荐。"
LangString DESC_VC    ${LANG_JAPANESE}     "ARKサーバーに必要なMicrosoft C++ランタイム。初回インストール時に推奨。"
LangString DESC_VC    ${LANG_KOREAN}       "ARK 서버에 필요한 Microsoft C++ 런타임. 첫 설치 시 권장."

LangString DESC_STEAM ${LANG_ENGLISH}      "Valve's tool to download Steam game servers. Needed to download ARK."
LangString DESC_STEAM ${LANG_SPANISH}      "Herramienta de Valve para descargar servidores de Steam. Necesario para descargar ARK."
LangString DESC_STEAM ${LANG_FRENCH}       "Outil Valve pour telecharger des serveurs Steam. Necessaire pour ARK."
LangString DESC_STEAM ${LANG_GERMAN}       "Valves Tool zum Herunterladen von Steam-Servern. Benoetigt fuer ARK-Download."
LangString DESC_STEAM ${LANG_ITALIAN}      "Tool di Valve per scaricare server Steam. Necessario per scaricare ARK."
LangString DESC_STEAM ${LANG_PORTUGUESEBR} "Ferramenta da Valve para baixar servidores Steam. Necessario para baixar ARK."
LangString DESC_STEAM ${LANG_RUSSIAN}      "Valve's tool to download Steam game servers. Needed to download ARK."
LangString DESC_STEAM ${LANG_SIMPCHINESE}  "Valve用于下载Steam服务器的工具。下载ARK服务器所需。"
LangString DESC_STEAM ${LANG_JAPANESE}     "SteamサーバーをダウンロードするValveのツール。ARKに必要。"
LangString DESC_STEAM ${LANG_KOREAN}       "Steam 서버를 다운로드하는 Valve 도구. ARK 다운로드에 필요."

LangString DESC_ARK   ${LANG_ENGLISH}      "Downloads the ARK dedicated server (~100 GB). Requires good internet. Can be done later."
LangString DESC_ARK   ${LANG_SPANISH}      "Descarga el servidor dedicado ARK (~100 GB). Requiere buena conexion. Puedes hacerlo despues."
LangString DESC_ARK   ${LANG_FRENCH}       "Telecharge le serveur ARK (~100 Go). Necessite une bonne connexion. Peut etre fait plus tard."
LangString DESC_ARK   ${LANG_GERMAN}       "Laedt den ARK-Server (~100 GB). Gute Verbindung erforderlich. Kann spaeter nachgeholt werden."
LangString DESC_ARK   ${LANG_ITALIAN}      "Scarica il server ARK (~100 GB). Richiede buona connessione. Puoi farlo dopo."
LangString DESC_ARK   ${LANG_PORTUGUESEBR} "Baixa o servidor ARK (~100 GB). Requer boa conexao. Pode ser feito depois."
LangString DESC_ARK   ${LANG_RUSSIAN}      "Downloads the ARK dedicated server (~100 GB). Requires good internet. Can be done later."
LangString DESC_ARK   ${LANG_SIMPCHINESE}  "下载ARK专用服务器（~100 GB）。需要良好网络。可稍后在应用中完成。"
LangString DESC_ARK   ${LANG_JAPANESE}     "ARK専用サーバーをダウンロード（~100 GB）。後でアプリから実行可。"
LangString DESC_ARK   ${LANG_KOREAN}       "ARK 전용 서버 다운로드（~100 GB）. 나중에 앱에서 할 수 있습니다."

; ─── MESSAGES ─────────────────────────────────────────────────────────────────
LangString MSG_DL_APP_ERR   ${LANG_ENGLISH}      "Could not download the app.$\r$\nCheck your internet connection.$\r$\nError: $0"
LangString MSG_DL_APP_ERR   ${LANG_SPANISH}      "No se pudo descargar la aplicacion.$\r$\nVerifica tu conexion a internet.$\r$\nError: $0"
LangString MSG_DL_APP_ERR   ${LANG_FRENCH}       "Impossible de telecharger l'application.$\r$\nVerifiez votre connexion.$\r$\nErreur: $0"
LangString MSG_DL_APP_ERR   ${LANG_GERMAN}       "App konnte nicht heruntergeladen werden.$\r$\nInternetverbindung pruefen.$\r$\nFehler: $0"
LangString MSG_DL_APP_ERR   ${LANG_ITALIAN}      "Impossibile scaricare l'app.$\r$\nControlla la connessione.$\r$\nErrore: $0"
LangString MSG_DL_APP_ERR   ${LANG_PORTUGUESEBR} "Nao foi possivel baixar o app.$\r$\nVerifique sua conexao.$\r$\nErro: $0"
LangString MSG_DL_APP_ERR   ${LANG_RUSSIAN}      "Could not download the app.$\r$\nCheck your internet connection.$\r$\nError: $0"
LangString MSG_DL_APP_ERR   ${LANG_SIMPCHINESE}  "无法下载应用程序。$\r$\n请检查您的网络连接。$\r$\n错误: $0"
LangString MSG_DL_APP_ERR   ${LANG_JAPANESE}     "アプリをダウンロードできませんでした。$\r$\n接続を確認してください。$\r$\nエラー: $0"
LangString MSG_DL_APP_ERR   ${LANG_KOREAN}       "앱을 다운로드할 수 없습니다.$\r$\n인터넷 연결을 확인하세요.$\r$\n오류: $0"

LangString MSG_ARK_PARTIAL  ${LANG_ENGLISH}      "Server download finished with code $0.$\r$\n$\r$\nThis may happen if download was interrupted.$\r$\nYou can retry from within the app at any time."
LangString MSG_ARK_PARTIAL  ${LANG_SPANISH}      "La descarga del servidor termino con codigo $0.$\r$\n$\r$\nEsto puede ocurrir si la descarga se interrumpio.$\r$\nPuedes reintentar desde la aplicacion."
LangString MSG_ARK_PARTIAL  ${LANG_FRENCH}       "Telechargement termine avec le code $0.$\r$\n$\r$\nCela peut arriver si interrompu.$\r$\nVous pouvez reessayer depuis l'application."
LangString MSG_ARK_PARTIAL  ${LANG_GERMAN}       "Download mit Code $0 beendet.$\r$\n$\r$\nPassiert wenn Download unterbrochen wurde.$\r$\nSie koennen es in der App wiederholen."
LangString MSG_ARK_PARTIAL  ${LANG_ITALIAN}      "Download terminato con codice $0.$\r$\n$\r$\nPuo accadere se il download e stato interrotto.$\r$\nPuoi riprovare dall'app."
LangString MSG_ARK_PARTIAL  ${LANG_PORTUGUESEBR} "Download concluido com codigo $0.$\r$\n$\r$\nPode ocorrer se o download foi interrompido.$\r$\nVoce pode tentar novamente pelo app."
LangString MSG_ARK_PARTIAL  ${LANG_RUSSIAN}      "Server download finished with code $0.$\r$\n$\r$\nThis may happen if download was interrupted.$\r$\nYou can retry from within the app at any time."
LangString MSG_ARK_PARTIAL  ${LANG_SIMPCHINESE}  "服务器下载以代码 $0 结束。$\r$\n$\r$\n如果下载被中断可能发生此情况。$\r$\n您可以随时从应用程序内重试。"
LangString MSG_ARK_PARTIAL  ${LANG_JAPANESE}     "ダウンロードがコード $0 で終了しました。$\r$\n$\r$\n中断された場合に起こる可能性があります。$\r$\nいつでもアプリから再試行できます。"
LangString MSG_ARK_PARTIAL  ${LANG_KOREAN}       "서버 다운로드가 코드 $0 으로 완료되었습니다.$\r$\n$\r$\n다운로드가 중단된 경우 발생할 수 있습니다.$\r$\n언제든지 앱에서 다시 시도할 수 있습니다."

LangString MSG_NO_STEAMCMD  ${LANG_ENGLISH}      "SteamCMD is not installed.$\r$\nCould not download ARK server.$\r$\n$\r$\nInstall SteamCMD in C:\ASA\steamcmd\ first."
LangString MSG_NO_STEAMCMD  ${LANG_SPANISH}      "SteamCMD no esta instalado.$\r$\nNo se pudo descargar el servidor ARK.$\r$\n$\r$\nInstala SteamCMD en C:\ASA\steamcmd\ primero."
LangString MSG_NO_STEAMCMD  ${LANG_FRENCH}       "SteamCMD n'est pas installe.$\r$\nImpossible de telecharger le serveur ARK.$\r$\n$\r$\nInstallez SteamCMD dans C:\ASA\steamcmd\ d'abord."
LangString MSG_NO_STEAMCMD  ${LANG_GERMAN}       "SteamCMD ist nicht installiert.$\r$\nARK-Server konnte nicht heruntergeladen werden.$\r$\n$\r$\nBitte zuerst SteamCMD in C:\ASA\steamcmd\ installieren."
LangString MSG_NO_STEAMCMD  ${LANG_ITALIAN}      "SteamCMD non e installato.$\r$\nImpossibile scaricare il server ARK.$\r$\n$\r$\nInstalla SteamCMD in C:\ASA\steamcmd\ prima."
LangString MSG_NO_STEAMCMD  ${LANG_PORTUGUESEBR} "SteamCMD nao esta instalado.$\r$\nNao foi possivel baixar o servidor ARK.$\r$\n$\r$\nInstale o SteamCMD em C:\ASA\steamcmd\ primeiro."
LangString MSG_NO_STEAMCMD  ${LANG_RUSSIAN}      "SteamCMD is not installed.$\r$\nCould not download ARK server.$\r$\n$\r$\nInstall SteamCMD in C:\ASA\steamcmd\ first."
LangString MSG_NO_STEAMCMD  ${LANG_SIMPCHINESE}  "未安装SteamCMD。$\r$\n无法下载ARK服务器。$\r$\n$\r$\n请先在 C:\ASA\steamcmd\ 安装SteamCMD。"
LangString MSG_NO_STEAMCMD  ${LANG_JAPANESE}     "SteamCMDがインストールされていません。$\r$\nARKサーバーをダウンロードできませんでした。$\r$\n$\r$\nまずC:\ASA\steamcmd\にSteamCMDをインストールしてください。"
LangString MSG_NO_STEAMCMD  ${LANG_KOREAN}       "SteamCMD가 설치되어 있지 않습니다.$\r$\nARK 서버를 다운로드할 수 없습니다.$\r$\n$\r$\n먼저 C:\ASA\steamcmd\에 SteamCMD를 설치하세요."

LangString MSG_64BIT        ${LANG_ENGLISH}      "This installer requires a 64-bit version of Windows."
LangString MSG_64BIT        ${LANG_SPANISH}      "Este instalador requiere Windows de 64 bits."
LangString MSG_64BIT        ${LANG_FRENCH}       "Cet installateur necessite une version 64 bits de Windows."
LangString MSG_64BIT        ${LANG_GERMAN}       "Dieses Installationsprogramm erfordert eine 64-Bit-Version von Windows."
LangString MSG_64BIT        ${LANG_ITALIAN}      "Questo programma richiede una versione a 64 bit di Windows."
LangString MSG_64BIT        ${LANG_PORTUGUESEBR} "Este instalador requer uma versao de 64 bits do Windows."
LangString MSG_64BIT        ${LANG_RUSSIAN}      "This installer requires a 64-bit version of Windows."
LangString MSG_64BIT        ${LANG_SIMPCHINESE}  "此安装程序需要64位版本的Windows。"
LangString MSG_64BIT        ${LANG_JAPANESE}     "このインストーラーは64ビット版のWindowsが必要です。"
LangString MSG_64BIT        ${LANG_KOREAN}       "이 설치 프로그램은 64비트 Windows가 필요합니다."

LangString MSG_WINVER       ${LANG_ENGLISH}      "Windows 10 or higher is required.$\r$\nPlease update your operating system."
LangString MSG_WINVER       ${LANG_SPANISH}      "Se requiere Windows 10 o superior.$\r$\nActualiza tu sistema operativo."
LangString MSG_WINVER       ${LANG_FRENCH}       "Windows 10 ou superieur est requis.$\r$\nMettez a jour votre systeme."
LangString MSG_WINVER       ${LANG_GERMAN}       "Windows 10 oder hoeher ist erforderlich.$\r$\nBitte aktualisieren Sie Ihr Betriebssystem."
LangString MSG_WINVER       ${LANG_ITALIAN}      "E richiesto Windows 10 o superiore.$\r$\nAggiorna il sistema operativo."
LangString MSG_WINVER       ${LANG_PORTUGUESEBR} "Windows 10 ou superior e necessario.$\r$\nAtualize seu sistema operacional."
LangString MSG_WINVER       ${LANG_RUSSIAN}      "Windows 10 or higher is required.$\r$\nPlease update your operating system."
LangString MSG_WINVER       ${LANG_SIMPCHINESE}  "需要Windows 10或更高版本。$\r$\n请更新您的操作系统。"
LangString MSG_WINVER       ${LANG_JAPANESE}     "Windows 10以降が必要です。$\r$\nOSを更新してください。"
LangString MSG_WINVER       ${LANG_KOREAN}       "Windows 10 이상이 필요합니다.$\r$\n운영 체제를 업데이트하십시오."

LangString MSG_UNINSTALL_ASA  ${LANG_ENGLISH}      "Do you also want to delete the ARK server and SteamCMD from C:\ASA\?$\r$\n$\r$\nWARNING: This will delete all saved worlds."
LangString MSG_UNINSTALL_ASA  ${LANG_SPANISH}      "Deseas eliminar tambien el servidor ARK y SteamCMD de C:\ASA\?$\r$\n$\r$\nATENCION: Esto eliminara todos los mundos guardados."
LangString MSG_UNINSTALL_ASA  ${LANG_FRENCH}       "Voulez-vous aussi supprimer le serveur ARK de C:\ASA\?$\r$\n$\r$\nATTENTION: Toutes les sauvegardes seront supprimees."
LangString MSG_UNINSTALL_ASA  ${LANG_GERMAN}       "Moechten Sie auch den ARK-Server von C:\ASA\ loeschen?$\r$\n$\r$\nWARNUNG: Alle gespeicherten Welten werden geloescht."
LangString MSG_UNINSTALL_ASA  ${LANG_ITALIAN}      "Vuoi eliminare anche il server ARK da C:\ASA\?$\r$\n$\r$\nATTENZIONE: Tutti i mondi salvati verranno eliminati."
LangString MSG_UNINSTALL_ASA  ${LANG_PORTUGUESEBR} "Deseja tambem excluir o servidor ARK de C:\ASA\?$\r$\n$\r$\nATENCAO: Todos os mundos salvos serao excluidos."
LangString MSG_UNINSTALL_ASA  ${LANG_RUSSIAN}      "Do you also want to delete the ARK server from C:\ASA\?$\r$\n$\r$\nWARNING: This will delete all saved worlds."
LangString MSG_UNINSTALL_ASA  ${LANG_SIMPCHINESE}  "您还想从 C:\ASA\ 删除ARK服务器吗？$\r$\n$\r$\n警告：这将删除所有保存的世界。"
LangString MSG_UNINSTALL_ASA  ${LANG_JAPANESE}     "C:\ASA\からARKサーバーも削除しますか？$\r$\n$\r$\n警告: すべての保存済みワールドが削除されます。"
LangString MSG_UNINSTALL_ASA  ${LANG_KOREAN}       "C:\ASA\에서 ARK 서버도 삭제하시겠습니까?$\r$\n$\r$\n경고: 모든 저장된 세계가 삭제됩니다."

LangString MSG_UNINSTALL_DONE ${LANG_ENGLISH}      "ARK ASA Configuration Manager has been uninstalled."
LangString MSG_UNINSTALL_DONE ${LANG_SPANISH}      "ARK ASA Configuration Manager ha sido desinstalado."
LangString MSG_UNINSTALL_DONE ${LANG_FRENCH}       "ARK ASA Configuration Manager a ete desinstalle."
LangString MSG_UNINSTALL_DONE ${LANG_GERMAN}       "ARK ASA Configuration Manager wurde deinstalliert."
LangString MSG_UNINSTALL_DONE ${LANG_ITALIAN}      "ARK ASA Configuration Manager e stato disinstallato."
LangString MSG_UNINSTALL_DONE ${LANG_PORTUGUESEBR} "ARK ASA Configuration Manager foi desinstalado."
LangString MSG_UNINSTALL_DONE ${LANG_RUSSIAN}      "ARK ASA Configuration Manager has been uninstalled."
LangString MSG_UNINSTALL_DONE ${LANG_SIMPCHINESE}  "ARK ASA Configuration Manager已被卸载。"
LangString MSG_UNINSTALL_DONE ${LANG_JAPANESE}     "ARK ASA Configuration Managerがアンインストールされました。"
LangString MSG_UNINSTALL_DONE ${LANG_KOREAN}       "ARK ASA Configuration Manager가 제거되었습니다."

; ─── Global vars ──────────────────────────────────────────────────────────────
Var CreateDesktopShortcut
Var CreateStartMenuShortcut

; ─── Custom shortcut page ─────────────────────────────────────────────────────
Function ShortcutPage
    nsDialogs::Create 1018
    Pop $0
    ${If} $0 == error
        Abort
    ${EndIf}
    ${NSD_CreateLabel}   0    0   100% 24u "$(SHORTCUT_LABEL)"
    Pop $0
    ${NSD_CreateCheckbox} 16u 32u 100% 12u "$(SHORTCUT_DESKTOP)"
    Pop $CreateDesktopShortcut
    ${NSD_SetState} $CreateDesktopShortcut ${BST_CHECKED}
    ${NSD_CreateCheckbox} 16u 50u 100% 12u "$(SHORTCUT_STARTMENU)"
    Pop $CreateStartMenuShortcut
    ${NSD_SetState} $CreateStartMenuShortcut ${BST_CHECKED}
    nsDialogs::Show
FunctionEnd

Function ShortcutPageLeave
    ${NSD_GetState} $CreateDesktopShortcut  $CreateDesktopShortcut
    ${NSD_GetState} $CreateStartMenuShortcut $CreateStartMenuShortcut
FunctionEnd

; ─── Section 1: App (required) ────────────────────────────────────────────────
Section "$(SEC_APP_NAME)" SEC_APP
    SectionIn RO
    SetOutPath "$INSTDIR"
    CreateDirectory "$INSTDIR"

    DetailPrint "Downloading ARK ASA Configuration Manager..."
    inetc::get /CAPTION "Downloading ARK ASA Config Manager..." \
        /BANNER "Please wait..." \
        "https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/download/v1.4/ARK.ASA.Config.Manager_2.0.0_x64-setup.exe" \
        "$TEMP\ark-app-setup.exe" /END
    Pop $0
    ${If} $0 != "OK"
        MessageBox MB_ICONEXCLAMATION|MB_OK "$(MSG_DL_APP_ERR)"
        Abort
    ${EndIf}

    DetailPrint "Installing ARK ASA Configuration Manager..."
    ExecWait '"$TEMP\ark-app-setup.exe" /S' $0
    Delete "$TEMP\ark-app-setup.exe"

    WriteUninstaller "$INSTDIR\Uninstall.exe"

    WriteRegStr   HKLM "Software\ARKASAConfigManager" "InstallDir" "$INSTDIR"
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" "DisplayName"     "ARK ASA Configuration Manager"
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" "DisplayVersion"  "1.3"
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" "Publisher"       "ARK ASA Config Manager"
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" "URLInfoAbout"    "https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" "NoRepair" 1

    ${If} $CreateDesktopShortcut == ${BST_CHECKED}
        CreateShortcut "$DESKTOP\ARK ASA Config Manager.lnk" \
            "$PROGRAMFILES64\ARK ASA Config Manager\ark-asa-config.exe"
    ${EndIf}
    ${If} $CreateStartMenuShortcut == ${BST_CHECKED}
        CreateDirectory "$SMPROGRAMS\ARK ASA Config Manager"
        CreateShortcut "$SMPROGRAMS\ARK ASA Config Manager\ARK ASA Config Manager.lnk" \
            "$PROGRAMFILES64\ARK ASA Config Manager\ark-asa-config.exe"
        CreateShortcut "$SMPROGRAMS\ARK ASA Config Manager\Uninstall.lnk" \
            "$INSTDIR\Uninstall.exe"
    ${EndIf}
SectionEnd

; ─── Section 2: Visual C++ ────────────────────────────────────────────────────
Section "$(SEC_VC_NAME)" SEC_VCREDIST
    ReadRegDWORD $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64" "Installed"
    ${If} $0 == 1
        DetailPrint "Visual C++ Redistributable already installed, skipping..."
    ${Else}
        DetailPrint "Downloading Visual C++ Redistributable 2022..."
        inetc::get /CAPTION "Downloading Visual C++ 2022..." \
            "https://aka.ms/vs/17/release/vc_redist.x64.exe" \
            "$TEMP\vc_redist.x64.exe" /END
        Pop $0
        ${If} $0 == "OK"
            ExecWait '"$TEMP\vc_redist.x64.exe" /install /quiet /norestart' $0
            Delete "$TEMP\vc_redist.x64.exe"
            ${If} $0 == 0
                DetailPrint "Visual C++ Redistributable installed successfully."
            ${ElseIf} $0 == 3010
                DetailPrint "Visual C++ installed (restart required, you can do it later)."
            ${EndIf}
        ${EndIf}
    ${EndIf}
SectionEnd

; ─── Section 3: SteamCMD ──────────────────────────────────────────────────────
Section "$(SEC_STEAM_NAME)" SEC_STEAMCMD
    CreateDirectory "C:\ASA\steamcmd"
    ${If} ${FileExists} "C:\ASA\steamcmd\steamcmd.exe"
        DetailPrint "SteamCMD already installed in C:\ASA\steamcmd\"
    ${Else}
        DetailPrint "Downloading SteamCMD from Valve..."
        inetc::get /CAPTION "Downloading SteamCMD..." \
            "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip" \
            "$TEMP\steamcmd.zip" /END
        Pop $0
        ${If} $0 == "OK"
            DetailPrint "Extracting SteamCMD to C:\ASA\steamcmd\..."
            nsExec::ExecToLog 'powershell -Command "Expand-Archive -Path \"$TEMP\steamcmd.zip\" -DestinationPath \"C:\ASA\steamcmd\" -Force"'
            Delete "$TEMP\steamcmd.zip"
        ${Else}
            DetailPrint "WARNING: SteamCMD download failed: $0"
        ${EndIf}
    ${EndIf}
    ${If} ${FileExists} "C:\ASA\steamcmd\steamcmd.exe"
        DetailPrint "Running SteamCMD first-time update (~30 seconds)..."
        nsExec::ExecToLog '"C:\ASA\steamcmd\steamcmd.exe" +quit'
        DetailPrint "SteamCMD ready."
    ${EndIf}
SectionEnd

; ─── Section 4: ARK Server (~100 GB) ──────────────────────────────────────────
Section "$(SEC_ARK_NAME)" SEC_ARKSERVER
    ${If} ${FileExists} "C:\ASA\steamcmd\steamcmd.exe"
        CreateDirectory "C:\ASA\server"
        DetailPrint "Starting ARK server download (App ID: 2430930)..."
        DetailPrint "This can take 1-2 hours. Do not close this window."
        ExecWait '"C:\ASA\steamcmd\steamcmd.exe" +force_install_dir "C:\ASA\server" +login anonymous +app_update 2430930 validate +quit' $0
        ${If} $0 == 0
            DetailPrint "ARK server downloaded and installed successfully."
            DetailPrint "Location: C:\ASA\server\"
        ${Else}
            MessageBox MB_ICONINFORMATION|MB_OK "$(MSG_ARK_PARTIAL)"
        ${EndIf}
    ${Else}
        MessageBox MB_ICONEXCLAMATION|MB_OK "$(MSG_NO_STEAMCMD)"
    ${EndIf}
SectionEnd

; ─── Section descriptions (tooltip) ──────────────────────────────────────────
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_APP}       "$(DESC_APP)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_VCREDIST}  "$(DESC_VC)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_STEAMCMD}  "$(DESC_STEAM)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_ARKSERVER} "$(DESC_ARK)"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; ─── Uninstaller ──────────────────────────────────────────────────────────────
Section "Uninstall"
    RMDir /r "$INSTDIR"
    Delete "$DESKTOP\ARK ASA Config Manager.lnk"
    RMDir /r "$SMPROGRAMS\ARK ASA Config Manager"
    DeleteRegKey HKLM "Software\ARKASAConfigManager"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager"
    MessageBox MB_ICONINFORMATION|MB_YESNO "$(MSG_UNINSTALL_ASA)" IDNO skip_delete_asa
    RMDir /r "C:\ASA"
    skip_delete_asa:
    MessageBox MB_ICONINFORMATION|MB_OK "$(MSG_UNINSTALL_DONE)"
SectionEnd

; ─── Init: language picker + OS checks ────────────────────────────────────────
Function .onInit
    !insertmacro MUI_LANGDLL_DISPLAY
    ${IfNot} ${RunningX64}
        MessageBox MB_ICONSTOP|MB_OK "$(MSG_64BIT)"
        Abort
    ${EndIf}
    ${IfNot} ${AtLeastWin10}
        MessageBox MB_ICONSTOP|MB_OK "$(MSG_WINVER)"
        Abort
    ${EndIf}
FunctionEnd
