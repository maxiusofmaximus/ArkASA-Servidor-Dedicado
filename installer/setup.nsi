; ============================================================
; ARK ASA Configuration Manager - Setup Completo
; Instala: App + VC++ + SteamCMD + Servidor ARK (~100 GB)
;
; Compilar con: makensis setup.nsi
; Requiere: NSIS 3.x + plugin inetc (incluido en NSIS Plugins)
; ============================================================

Unicode True
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "Sections.nsh"
!include "WinMessages.nsh"

; ─── Metadatos ────────────────────────────────────────────
Name "ARK ASA Configuration Manager"
OutFile "ARK-ASA-Full-Setup.exe"
InstallDir "C:\Program Files\ARK ASA Config Manager"
InstallDirRegKey HKLM "Software\ARKASAConfigManager" "InstallDir"
RequestExecutionLevel admin

VIProductVersion "1.2.0.0"
VIAddVersionKey "ProductName"     "ARK ASA Configuration Manager Setup"
VIAddVersionKey "ProductVersion"  "1.2.0"
VIAddVersionKey "FileDescription" "Instalador completo para ARK ASA Config Manager"
VIAddVersionKey "LegalCopyright"  "MIT License - 2026"
VIAddVersionKey "CompanyName"     "ARK ASA Config Manager"

; ─── Íconos ───────────────────────────────────────────────
!define MUI_ICON   "..\src-tauri\icons\icon.ico"
!define MUI_UNICON "..\src-tauri\icons\icon.ico"

; ─── Colores/Estilo ───────────────────────────────────────
!define MUI_ABORTWARNING
!define MUI_ABORTWARNING_TEXT "¿Seguro que quieres cancelar la instalación?"
!define MUI_ABORTWARNING_CANCEL_DEFAULT

; ─── Textos de bienvenida ─────────────────────────────────
!define MUI_WELCOMEPAGE_TITLE     "Bienvenido a ARK ASA Config Manager"
!define MUI_WELCOMEPAGE_TEXT      "Este asistente instalará todo lo necesario para tener tu \
servidor de ARK: Survival Ascended funcionando:$\r$\n$\r$\n\
  • ARK ASA Configuration Manager$\r$\n\
  • Visual C++ Redistributable 2022$\r$\n\
  • SteamCMD (herramienta de Valve)$\r$\n\
  • Servidor dedicado ARK: Survival Ascended$\r$\n$\r$\n\
⚠  La descarga del servidor requiere aprox. $\r$\n\
   100 GB de espacio libre y 1-2 horas según$\r$\n\
   tu conexión a internet.$\r$\n$\r$\n\
Haz clic en Siguiente para continuar."

; ─── Página de finalización ───────────────────────────────
!define MUI_FINISHPAGE_RUN          "$INSTDIR\ark-asa-config.exe"
!define MUI_FINISHPAGE_RUN_TEXT     "Iniciar ARK ASA Configuration Manager"
!define MUI_FINISHPAGE_LINK         "Ver documentación online"
!define MUI_FINISHPAGE_LINK_LOCATION "https://maxiusofmaximus.github.io/ArkASA-Servidor-Dedicado/"
!define MUI_FINISHPAGE_TITLE        "¡Instalación completada!"
!define MUI_FINISHPAGE_TEXT         "ARK ASA Configuration Manager y todos sus componentes \
han sido instalados correctamente.$\r$\n$\r$\n\
Tu servidor de ARK: Survival Ascended está listo.$\r$\n\
Abre la aplicación para configurarlo e iniciarlo."

; ─── Páginas del asistente ────────────────────────────────
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "license.txt"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
Page custom ShortcutPage ShortcutPageLeave
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Desinstalador
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "Spanish"

; ─── Variables globales ───────────────────────────────────
Var CreateDesktopShortcut
Var CreateStartMenuShortcut

; ─── Página personalizada: Accesos directos ───────────────
Function ShortcutPage
    nsDialogs::Create 1018
    Pop $0
    ${If} $0 == error
        Abort
    ${EndIf}

    ${NSD_CreateLabel} 0 0 100% 24u "Selecciona dónde crear accesos directos:"
    Pop $0

    ${NSD_CreateCheckbox} 16u 32u 100% 12u "Crear acceso directo en el Escritorio"
    Pop $CreateDesktopShortcut
    ${NSD_SetState} $CreateDesktopShortcut ${BST_CHECKED}

    ${NSD_CreateCheckbox} 16u 50u 100% 12u "Crear acceso directo en el Menú Inicio"
    Pop $CreateStartMenuShortcut
    ${NSD_SetState} $CreateStartMenuShortcut ${BST_CHECKED}

    nsDialogs::Show
FunctionEnd

Function ShortcutPageLeave
    ${NSD_GetState} $CreateDesktopShortcut  $CreateDesktopShortcut
    ${NSD_GetState} $CreateStartMenuShortcut $CreateStartMenuShortcut
FunctionEnd

; ─── Secciones de instalación ─────────────────────────────

; SECCIÓN 1: App principal (obligatoria)
Section "ARK ASA Config Manager (requerido)" SEC_APP
    SectionIn RO  ; No se puede desmarcar
    DetailPrint "━━ Instalando ARK ASA Configuration Manager ━━"

    SetOutPath "$INSTDIR"

    ; Crear directorio de instalación
    CreateDirectory "$INSTDIR"

    ; Descargar el instalador de la app desde GitHub Releases
    DetailPrint "Descargando ARK ASA Configuration Manager..."
    inetc::get /CAPTION "Descargando aplicación..." \
        /BANNER "Esto puede tardar un momento..." \
        "https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado/releases/download/v1.1/ARK.ASA.Config.Manager_2.0.0_x64-setup.exe" \
        "$TEMP\ark-app-setup.exe" /END
    Pop $0
    ${If} $0 != "OK"
        DetailPrint "⚠ Error descargando la app: $0"
        MessageBox MB_ICONEXCLAMATION|MB_OK "No se pudo descargar la aplicación.$\r$\nVerifica tu conexión a internet.$\r$\nError: $0"
        Abort
    ${EndIf}

    DetailPrint "Instalando ARK ASA Configuration Manager..."
    ExecWait '"$TEMP\ark-app-setup.exe" /S' $0
    ${If} $0 != 0
        DetailPrint "⚠ El instalador de la app terminó con código: $0"
    ${Else}
        DetailPrint "✓ ARK ASA Config Manager instalado correctamente"
    ${EndIf}

    ; Copiar launcher al directorio de instalación
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Registro de Windows (para Agregar/Quitar programas)
    WriteRegStr   HKLM "Software\ARKASAConfigManager" "InstallDir" "$INSTDIR"
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" \
                       "DisplayName" "ARK ASA Configuration Manager"
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" \
                       "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" \
                       "DisplayVersion" "1.1"
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" \
                       "Publisher" "ARK ASA Config Manager"
    WriteRegStr   HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" \
                       "URLInfoAbout" "https://github.com/maxiusofmaximus/ArkASA-Servidor-Dedicado"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" \
                       "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager" \
                       "NoRepair" 1

    ; Accesos directos
    ${If} $CreateDesktopShortcut == ${BST_CHECKED}
        CreateShortcut "$DESKTOP\ARK ASA Config Manager.lnk" \
            "$PROGRAMFILES64\ARK ASA Config Manager\ark-asa-config.exe" "" \
            "$PROGRAMFILES64\ARK ASA Config Manager\ark-asa-config.exe" 0
        DetailPrint "✓ Acceso directo creado en el Escritorio"
    ${EndIf}

    ${If} $CreateStartMenuShortcut == ${BST_CHECKED}
        CreateDirectory "$SMPROGRAMS\ARK ASA Config Manager"
        CreateShortcut "$SMPROGRAMS\ARK ASA Config Manager\ARK ASA Config Manager.lnk" \
            "$PROGRAMFILES64\ARK ASA Config Manager\ark-asa-config.exe" "" \
            "$PROGRAMFILES64\ARK ASA Config Manager\ark-asa-config.exe" 0
        CreateShortcut "$SMPROGRAMS\ARK ASA Config Manager\Desinstalar.lnk" \
            "$INSTDIR\Uninstall.exe"
        DetailPrint "✓ Acceso directo creado en el Menú Inicio"
    ${EndIf}

    DetailPrint "✓ Sección App completada"
SectionEnd

; SECCIÓN 2: Visual C++ Runtime
Section "Visual C++ Redistributable 2022 (recomendado)" SEC_VCREDIST
    DetailPrint "━━ Instalando Visual C++ Redistributable 2022 ━━"

    ; Verificar si ya está instalado
    ReadRegDWORD $0 HKLM \
        "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64" "Installed"
    ${If} $0 == 1
        DetailPrint "✓ Visual C++ Redistributable ya está instalado, omitiendo..."
    ${Else}
        DetailPrint "Descargando Visual C++ Redistributable 2022..."
        inetc::get /CAPTION "Descargando Visual C++ 2022..." \
            "https://aka.ms/vs/17/release/vc_redist.x64.exe" \
            "$TEMP\vc_redist.x64.exe" /END
        Pop $0
        ${If} $0 != "OK"
            DetailPrint "⚠ Error descargando VC++: $0 — Continuando sin él..."
        ${Else}
            DetailPrint "Instalando Visual C++ Redistributable..."
            ExecWait '"$TEMP\vc_redist.x64.exe" /install /quiet /norestart' $0
            ${If} $0 == 0
                DetailPrint "✓ Visual C++ Redistributable instalado"
            ${ElseIf} $0 == 3010
                DetailPrint "✓ Visual C++ instalado (se requiere reinicio, puedes hacerlo después)"
            ${Else}
                DetailPrint "⚠ VC++ terminó con código $0 — puede que ya estuviera instalado"
            ${EndIf}
            Delete "$TEMP\vc_redist.x64.exe"
        ${EndIf}
    ${EndIf}
SectionEnd

; SECCIÓN 3: SteamCMD
Section "SteamCMD (necesario para el servidor)" SEC_STEAMCMD
    DetailPrint "━━ Instalando SteamCMD ━━"

    CreateDirectory "C:\ASA\steamcmd"

    ; Verificar si ya existe
    ${If} ${FileExists} "C:\ASA\steamcmd\steamcmd.exe"
        DetailPrint "✓ SteamCMD ya está instalado en C:\ASA\steamcmd\"
    ${Else}
        DetailPrint "Descargando SteamCMD desde Valve..."
        inetc::get /CAPTION "Descargando SteamCMD..." \
            "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip" \
            "$TEMP\steamcmd.zip" /END
        Pop $0
        ${If} $0 != "OK"
            DetailPrint "⚠ Error descargando SteamCMD: $0"
            MessageBox MB_ICONEXCLAMATION|MB_RETRYCANCEL \
                "No se pudo descargar SteamCMD.$\r$\nError: $0$\r$\n$\r$\nSin SteamCMD no se puede instalar el servidor ARK." \
                IDRETRY retry_steamcmd IDCANCEL skip_steamcmd
            retry_steamcmd:
                inetc::get /CAPTION "Reintentando SteamCMD..." \
                    "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip" \
                    "$TEMP\steamcmd.zip" /END
                Pop $0
            skip_steamcmd:
        ${EndIf}

        ${If} $0 == "OK"
            DetailPrint "Extrayendo SteamCMD en C:\ASA\steamcmd\..."
            nsisunz::UnzipToLog "$TEMP\steamcmd.zip" "C:\ASA\steamcmd"
            Pop $0
            ${If} $0 != "success"
                ; Fallback: usar PowerShell para descomprimir
                DetailPrint "Usando PowerShell para extraer..."
                nsExec::ExecToLog 'powershell -Command "Expand-Archive -Path \"$TEMP\steamcmd.zip\" -DestinationPath \"C:\ASA\steamcmd\" -Force"'
            ${EndIf}
            Delete "$TEMP\steamcmd.zip"

            ${If} ${FileExists} "C:\ASA\steamcmd\steamcmd.exe"
                DetailPrint "✓ SteamCMD instalado en C:\ASA\steamcmd\"
            ${Else}
                DetailPrint "⚠ No se pudo verificar SteamCMD — Revisa C:\ASA\steamcmd\"
            ${EndIf}
        ${EndIf}
    ${EndIf}

    ; Primera ejecución de SteamCMD para que se actualice
    ${If} ${FileExists} "C:\ASA\steamcmd\steamcmd.exe"
        DetailPrint "Actualizando SteamCMD por primera vez (tarda ~30 segundos)..."
        nsExec::ExecToLog '"C:\ASA\steamcmd\steamcmd.exe" +quit'
        DetailPrint "✓ SteamCMD listo"
    ${EndIf}
SectionEnd

; SECCIÓN 4: Servidor ARK (la descarga grande)
Section "ARK: Survival Ascended - Servidor Dedicado (~100 GB)" SEC_ARKSERVER
    DetailPrint "━━ Descargando ARK: Survival Ascended Dedicated Server ━━"
    DetailPrint ""
    DetailPrint "⚠ ESTO PUEDE TARDAR 1-2 HORAS"
    DetailPrint "   No cierres esta ventana."
    DetailPrint ""

    ${If} ${FileExists} "C:\ASA\steamcmd\steamcmd.exe"
        CreateDirectory "C:\ASA\server"

        DetailPrint "Iniciando descarga del servidor ARK..."
        DetailPrint "App ID: 2430930 (ARK: Survival Ascended Dedicated Server)"
        DetailPrint ""
        DetailPrint "Se abrirá una ventana de SteamCMD mostrando el progreso..."
        DetailPrint "Cuando termine, aparecerá 'App 2430930 fully installed'."

        ExecWait '"C:\ASA\steamcmd\steamcmd.exe" \
            +force_install_dir "C:\ASA\server" \
            +login anonymous \
            +app_update 2430930 validate \
            +quit' $0

        ${If} $0 == 0
            DetailPrint ""
            DetailPrint "✓ Servidor ARK descargado e instalado correctamente"
            DetailPrint "   Ubicación: C:\ASA\server\"
        ${Else}
            DetailPrint ""
            DetailPrint "⚠ SteamCMD terminó con código: $0"
            DetailPrint "   Puede que la descarga haya quedado incompleta."
            DetailPrint "   Puedes volver a correr la descarga desde la app."
            MessageBox MB_ICONINFORMATION|MB_OK \
                "La descarga del servidor terminó con código $0.$\r$\n$\r$\n\
Esto puede ocurrir si la descarga se interrumpió o si SteamCMD tuvo$\r$\n\
un problema temporal.$\r$\n$\r$\n\
Puedes reintentar la descarga desde dentro de la aplicación en$\r$\n\
cualquier momento. No es necesario reinstalar todo."
        ${EndIf}
    ${Else}
        DetailPrint "⚠ SteamCMD no encontrado, omitiendo descarga del servidor."
        DetailPrint "   Instala SteamCMD primero y luego descarga el servidor desde la app."
        MessageBox MB_ICONEXCLAMATION|MB_OK \
            "SteamCMD no está instalado.$\r$\n\
No se pudo descargar el servidor ARK.$\r$\n$\r$\n\
Instala SteamCMD manualmente en C:\ASA\steamcmd\ \
y luego descarga el servidor desde la aplicación."
    ${EndIf}
SectionEnd

; ─── Descripciones de secciones (tooltip en la página de componentes) ───
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_APP}        "La aplicación de escritorio para gestionar y lanzar tu servidor ARK. Obligatorio."
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_VCREDIST}   "Librería de Microsoft necesaria para ejecutar aplicaciones compiladas en C++. Muy recomendado si es la primera vez."
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_STEAMCMD}   "Herramienta de Valve para descargar servidores de Steam. Necesario para descargar el servidor ARK."
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_ARKSERVER}  "Descarga el servidor dedicado de ARK: Survival Ascended (~100 GB). Requiere buena conexión. Puedes hacerlo después si lo prefieres."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; ─── Desinstalador ────────────────────────────────────────
Section "Uninstall"
    RMDir /r "$INSTDIR"

    Delete "$DESKTOP\ARK ASA Config Manager.lnk"
    RMDir /r "$SMPROGRAMS\ARK ASA Config Manager"

    DeleteRegKey HKLM "Software\ARKASAConfigManager"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ARKASAConfigManager"

    MessageBox MB_ICONINFORMATION|MB_YESNO \
        "¿Deseas eliminar también el servidor ARK y SteamCMD de C:\ASA\?$\r$\n$\r$\n\
⚠ Esto eliminará todos los mundos guardados." \
        IDNO skip_server_delete
    RMDir /r "C:\ASA"
    skip_server_delete:

    MessageBox MB_ICONINFORMATION|MB_OK "ARK ASA Configuration Manager ha sido desinstalado."
SectionEnd

; ─── Init: verificar arquitectura ─────────────────────────
Function .onInit
    ${IfNot} ${RunningX64}
        MessageBox MB_ICONSTOP|MB_OK \
            "Este instalador requiere Windows de 64 bits.$\r$\nNo es compatible con sistemas de 32 bits."
        Abort
    ${EndIf}

    ; Verificar Windows 10 mínimo
    ${If} ${AtLeastWin10}
        ; OK
    ${Else}
        MessageBox MB_ICONSTOP|MB_OK \
            "Se requiere Windows 10 o superior.$\r$\nActualiza tu sistema operativo."
        Abort
    ${EndIf}

    ; Verificar espacio en C:\ (necesita al menos 150 GB)
    SectionGetSize ${SEC_ARKSERVER} $0
    ; Comprobación básica de espacio libre
    StrCpy $0 "C:"
    System::Call 'kernel32::GetDiskFreeSpaceEx(t, *l, *l, *l) i (r0, .r1, .r2, .r3)'
FunctionEnd
