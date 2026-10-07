@echo off
REM ==========================================
REM Compilateur MSL5.0 - Setup et Compilation
REM ==========================================
REM Ce script automatise l'installation et la compilation du compilateur Rust
REM Il copie le compilateur dans Documents et ajoute le chemin au PATH Windows

setlocal enabledelayedexpansion

echo.
echo ========================================
echo  Compilateur MSL5.0 - Setup Automatique
echo ========================================
echo.

REM Récupérer le chemin Documents de l'utilisateur
set DOCUMENTS=%USERPROFILE%\Documents

REM Définir le chemin cible
set TARGET_PATH=%DOCUMENTS%\Compilateur\4.5\compilator

echo [*] Documents detecetes : %DOCUMENTS%
echo [*] Chemin cible : %TARGET_PATH%
echo.

REM Vérifier si le répertoire Documents existe
if not exist "%DOCUMENTS%" (
    echo [ERREUR] Le dossier Documents n'a pas pu etre localise.
    pause
    exit /b 1
)

REM Vérifier si le compilateur est déjà installé
if exist "%TARGET_PATH%" (
    echo [INFO] Compilateur deja installe a : %TARGET_PATH%
    echo.
) else (
    echo [ERREUR] Le dossier Compilateur n'a pas ete trouve dans Documents.
    echo [INFO] Assurez-vous d'avoir copie le dossier Compilateur dans : %DOCUMENTS%
    echo.
    pause
    exit /b 1
)

REM Ajouter le chemin du compilateur au PATH Windows
echo [*] Ajout du compilateur au PATH Windows...
setx PATH "%TARGET_PATH%;!PATH!"

if %ERRORLEVEL% equ 0 (
    echo [OK] Compilateur ajoute au PATH avec succes.
) else (
    echo [AVERTISSEMENT] Impossible d'ajouter au PATH. Vous devrez peut-etre le faire manuellement.
)
echo.

REM Compiler le compilateur Rust
echo [*] Compilation du compilateur Rust...
echo [*] Cela peut prendre quelques minutes...
echo.

cd /d "%TARGET_PATH%"

if not exist "Cargo.toml" (
    echo [ERREUR] Cargo.toml non trouve dans %TARGET_PATH%
    echo [INFO] Assurez-vous que la structure du dossier est intacte.
    pause
    exit /b 1
)

REM Vérifier si Rust et Cargo sont installes
cargo --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERREUR] Cargo n'a pas ete trouve. Veuillez installer Rust et Cargo.
    echo [INFO] Visitez : https://www.rust-lang.org/tools/install
    pause
    exit /b 1
)

REM Compiler
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERREUR] La compilation a echoue.
    pause
    exit /b 1
)

echo.
echo ========================================
echo [OK] Installation terminee avec succes !
echo ========================================
echo.
echo [INFO] Vous pouvez maintenant utiliser le compilateur depuis n'importe quel terminal.
echo.
echo [COMMANDE] Pour compiler un fichier MSL5 :
echo   compilator [chemin/vers/fichier.msl5] [nom_sortie.bin]
echo.
echo [EXEMPLE]
echo   compilator C:/Users/[Utilisateur]/Desktop/mon_programme.msl5 out.bin
echo.
echo [NOTE] Les fichiers OpC, D1, D2 seront generes dans :
echo   %TARGET_PATH%\
echo.
echo.
pause
