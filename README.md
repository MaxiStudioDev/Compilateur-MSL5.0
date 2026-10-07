# Histoire du MSL5

Si vous voulez savoir le début de cette histoire, la voici :
Version Amelliorer :
[https://maxistudiodev.github.io/Cpu-32-Bits-Project-MSL5P]
version classique:
[https://github.com/MaxiStudioDev/Cpu-32-Bits-Project-MSL5P]


# Compilateur MSL5.0

Un compilateur et un environnement de simulation pour le langage MSL5.0, écrit en Rust. Le projet permet de transformer des programmes MSL5 en instructions binaires exploitable par l'émulateur CPU 32-Bits via **AntaresCircuit.io**.

## Vue d'ensemble

MSL5.0 est un langage de programmation orienté logique/contrôle, avec des notions de registres, de conditions, de boucles et de GPIO. Ce dépôt contient :

- un compilateur Rust qui parse et transforme le code MSL5.0 en instructions encodées,
- un plugin VS Code pour faciliter l'écriture du langage,
- des exemples de programmes et des sorties compilées,
- une intégration complète avec l'émulateur CPU 32-Bits (AntaresCircuit.io),
- un script batch automatisé pour Windows (MSL5Compilateur.bat).

## Fonctionnalités

- Analyse syntaxique de scripts MSL5.0
- Support des variables et registres
- Gestion des conditions `IF`, `WHILE`, `LOOP`, `BREAK`
- Support des opérations ALU (`ADD`, `SUB`, `AND`, `OR`, `XOR`, etc.)
- Gestion de la logique d'entrée/sortie GPIO (`IN`, `OUT`, `IF IN`)
- Génération de fichiers binaire (`.bin`) et texte (`.txt`)
- Extension VS Code avec coloration syntaxique et snippets
- Architecture modulaire selon les différents composants du projet
- Intégration avec AntaresCircuit.io pour la simulation sur CPU 32-Bits

## Structure du dépôt

```text
Compilateur-MSL5.0/
├── README.md
├── Compilateur/
│   └── 4.5/
│       ├── compilator/          # Compilateur principal Rust
│       │   ├── src/
│       │   ├── Cargo.toml
│       │   ├── test.msl5
│       │   ├── out.bin
│       │   ├── out.txt
│       │   ├── OpC
│       │   ├── D1
│       │   └── D2
│       ├── PluginAU/            # Extension VS Code
│       ├── msl5-mini-pro-4.5.0.vsix
│       └── setup_and_compile.bat # Script d'installation automatique
└── ...
```

## Guide de démarrage rapide

### Étape 1 : Installer AntaresCircuit.io

1. Visitez [AntaresCircuit.io](https://antarescircuit.io)
2. Téléchargez et installez l'application
3. Lancez AntaresCircuit.io

### Étape 2 : Importer le fichier CPU 32-Bits

1. Téléchargez le fichier de configuration du CPU :
   ```
   https://raw.githubusercontent.com/MaxiStudioDev/Cpu-32-Bits-Project-MSL5P/refs/heads/main/CPUv5.0.acp
   ```

2. Ouvrez AntaresCircuit.io

3. Importez le fichier `CPUv5.0.acp` :
   - Cliquez sur **File** → **Import** (ou **Importer**)
   - Sélectionnez le fichier téléchargé
   - Confirmez l'importation

### Étape 3 : Configurer le compilateur

Vous avez deux options pour installer le compilateur MSL5.0 :

---

## Option 1 : Compiler depuis les sources Rust

### Pour utilisateurs avancés

**Prérequis :**
- Rust (dernière version stable)
- Cargo

**Installation :**

1. Clonez ou téléchargez ce repository
2. Copiez le dossier `Compilateur` complet dans votre dossier Documents :
   ```
   C:\Users\[VotreNomUtilisateur]\Documents\Compilateur\
   ```

3. Ouvrez un terminal et naviguez vers le compilateur :
   ```bash
   cd Documents/Compilateur/4.5/compilator
   ```

4. Compilez le compilateur Rust :
   ```bash
   cargo build --release
   ```

5. Testez l'installation :
   ```bash
   cargo run --release -- test.msl5 out.bin
   ```

Si tout fonctionne, vous pouvez maintenant compiler des programmes MSL5 depuis n'importe quel terminal.

---

## Option 2 : Utiliser le script automatisé (Windows x86_64 recommandé)

### Pour utilisateurs Windows x86_64

**⚠️ Disponible uniquement sur Windows x86_64. Autres architectures/OS : utilisez l'Option 1.**

C'est la méthode la plus simple et la plus rapide. Le script automatise tout le processus.

### Installation du script

1. Téléchargez le fichier batch automatisé :
   ```
   📥 MSL5Compilateur.bat
   https://raw.githubusercontent.com/MaxiStudioDev/Compilateur-MSL5.0/main/Compilateur/4.5/setup_and_compile.bat
   ```

2. Copiez le dossier `Compilateur` complet dans votre dossier Documents :
   ```
   C:\Users\[VotreNomUtilisateur]\Documents\Compilateur\
   ```

3. **Double-cliquez sur le fichier `MSL5Compilateur.bat`** ou exécutez-le depuis PowerShell

### Ce que le script fait automatiquement

Le script automatise les étapes suivantes :

- ✅ Localise votre dossier Documents
- ✅ Vérifie que le dossier `Compilateur` est correctement placé
- ✅ Vérifie que Rust et Cargo sont installés
- ✅ Compile le compilateur Rust en mode Release
- ✅ Ajoute le compilateur au PATH Windows
- ✅ Affiche les instructions de suivi

Après l'installation, vous pouvez utiliser la commande `compilator` depuis n'importe quel terminal.

### Utiliser le compilateur (après installation du script)

Une fois le script exécuté avec succès, vous pouvez compiler des fichiers MSL5 depuis n'importe quel terminal :

```bash
compilator C:/chemin/vers/fichier.msl5 out.bin
```

**Exemple :**

```bash
compilator C:/Users/MonUtilisateur/Desktop/mon_programme.msl5 out.bin
```

Les fichiers générés seront sauvegardés dans le même dossier que le programme compilé :
- `out.bin` : version binaire
- `out.txt` : version textuelle
- `OpC`, `D1`, `D2` : fichiers pour AntaresCircuit.io

---

## Utilisation du compilateur (toutes les options)

### Créer un programme MSL5

Créez un fichier avec l'extension `.msl5` :

```msl5
let x = 10
let y = 20

if x == y {
    out(1).set(1)
} else {
    out(1).set(0)
}
```

### Compiler le programme

**Option 1 (Rust / cargo) :**
```bash
cd Documents/Compilateur/4.5/compilator
cargo run --release -- /chemin/vers/mon_programme.msl5 out.bin
```

**Option 2 (Script batch - depuis n'importe où) :**
```bash
compilator C:/chemin/vers/mon_programme.msl5 out.bin
```

### Fichiers générés

Le compilateur génère les fichiers suivants dans le dossier `Compilateur/4.5/compilator/` :

- **`out.bin`** : version binaire du programme
- **`out.txt`** : représentation textuelle des instructions
- **`OpC`** : fichier pour l'émulateur
- **`D1`** : fichier pour l'émulateur
- **`D2`** : fichier pour l'émulateur

### Importer dans AntaresCircuit.io

1. Ouvrez AntaresCircuit.io avec le CPU 32-Bits (CPUv5.0.acp) importé
2. Chargez le fichier `out.bin` généré
3. Lancez la simulation pour visualiser le comportement de votre programme

---

## Installation du plugin VS Code (optionnel)

Pour bénéficier de la coloration syntaxique et des snippets MSL5.0 dans VS Code :

1. Ouvrez VS Code
2. Allez dans **Extensions** (Ctrl+Shift+X ou Cmd+Shift+X)
3. Installez l'extension locale `.vsix` :

   **Option A - Ligne de commande :**
   ```bash
   code --install-extension Documents/Compilateur/4.5/msl5-mini-pro-4.5.0.vsix
   ```

   **Option B - Interface graphique :**
   - Allez à **Extensions** → **...** (menu en haut à droite) → **Install from VSIX...**
   - Naviguez vers `Documents/Compilateur/4.5/msl5-mini-pro-4.5.0.vsix`
   - Confirmez l'installation

4. Redémarrez VS Code

Les fichiers `.msl5` seront désormais colorisés automatiquement avec :
- Coloration syntaxique complète
- Snippets de code
- Auto-complétion basique

---

## ⚠️ IMPORTANT - Structure du dossier Compilateur

### Emplacement obligatoire

**Le dossier `Compilateur` DOIT être obligatoirement situé dans votre dossier Documents :**

```
C:\Users\[VotreNomUtilisateur]\Documents\Compilateur\
```

**Pourquoi ?** L'émulateur AntaresCircuit.io et le projet Cpu-32-Bits-Project-MSL5P recherchent les fichiers (`OpC`, `D1`, `D2`) et le compilateur à cet emplacement exact. Si vous le placez ailleurs, l'émulateur ne trouvera pas les fichiers et cessera de fonctionner.

### Ne pas modifier l'arborescence

**Le dossier `Compilateur` et toute son arborescence DOIVENT rester intacts après l'installation :**

**Ne pas modifier, déplacer ou supprimer :**
- ✅ La structure complète du dossier `Compilateur/4.5/`
- ✅ Les fichiers de sortie du compilateur : `OpC`, `D1`, `D2`
- ✅ L'emplacement du compilateur Rust dans `Compilateur/4.5/compilator/`
- ✅ Tous les fichiers de configuration et de dépendances

**Pourquoi ?** L'émulateur AntaresCircuit.io dépend de ces fichiers aux emplacements spécifiques. Si vous modifiez l'arborescence ou déplacez les fichiers, l'émulateur ne pourra pas les localiser et cessera de fonctionner correctement.

### Les fichiers OpC, D1, D2 sont générés automatiquement

À chaque compilation, les fichiers `OpC`, `D1`, `D2` sont **générés et remplacés** automatiquement dans `Compilateur/4.5/compilator/`. Vous ne devez pas les modifier manuellement.

---

## Workflow complet : de la programmation à la simulation

1. **Écrivez votre code** dans VS Code (avec le plugin, optionnel)
2. **Compilez** avec la commande `compilator` ou `cargo run`
3. **Vérifiez les sorties** : `out.bin`, `out.txt`, `OpC`, `D1`, `D2`
4. **Ouvrez AntaresCircuit.io** avec le CPU 32-Bits (CPUv5.0.acp) importé
5. **Chargez** le fichier `out.bin` généré
6. **Lancez la simulation** pour visualiser le comportement de votre programme

---

## Dépannage

### Le compilateur ne compile pas
- Vérifiez que Rust et Cargo sont installés : `rustc --version` et `cargo --version`
- Vérifiez que le dossier `Compilateur` se trouve dans `Documents/Compilateur/`
- Vérifiez la structure du dossier : `Compilateur/4.5/compilator/Cargo.toml` doit exister

### Le script batch ne fonctionne pas
- Vérifiez que vous êtes sur **Windows x86_64**
- Vérifiez que **Rust est installé** : https://www.rust-lang.org/tools/install
- Lancez le script **en tant qu'administrateur**
- Redémarrez le terminal après l'installation du script

### AntaresCircuit.io ne trouve pas les fichiers
- Vérifiez que le dossier `Compilateur` est dans `Documents`
- Vérifiez que vous avez compilé au moins une fois
- Vérifiez que les fichiers `OpC`, `D1`, `D2` existent dans `Compilateur/4.5/compilator/`

---

## Prérequis

### Option 1 (Rust) :
- Rust (dernière version stable recommandée)
- Cargo
- Windows, Linux ou macOS

### Option 2 (Script batch) :
- Windows x86_64 uniquement
- Rust (dernière version stable recommandée)
- Cargo
- Terminal PowerShell, CMD ou Git Bash

### Pour l'extension VS Code :
- VS Code (optionnel)

---

## Exemple de programme complet

```msl5
let temperature = 50
let threshold = 45

if temperature > threshold {
    out(1).set(1)
    out(2).set(0)
} else {
    out(1).set(0)
    out(2).set(1)
}
```

---

## Développement

Le projet est principalement développé en Rust, avec des composants complémentaires en JavaScript pour l'éditeur de texte. Il est adapté à une utilisation académique, expérimentale ou de prototypage.

## Licence

Aucune licence explicite n'est indiquée dans le dépôt pour le moment. Vérifiez bien les droits avant une utilisation commerciale ou une redistribution.

## Contributions

Les contributions sont les bienvenues. N'hésitez pas à proposer des améliorations, des corrections de bugs ou de nouveaux exemples.

## Auteurs

Projet développé par MaxiStudioDev.

## Contact

Pour toute question ou suggestion, vous pouvez ouvrir une issue sur le dépôt GitHub ou contacter le propriétaire du projet.

### Liens utiles

- 🔗 [AntaresCircuit.io](https://antarescircuit.io)
- 🔗 [Cpu-32-Bits-Project-MSL5P](https://github.com/MaxiStudioDev/Cpu-32-Bits-Project-MSL5P)
- 🔗 [Repository Compilateur-MSL5.0](https://github.com/MaxiStudioDev/Compilateur-MSL5.0)
- 📥 [Télécharger MSL5Compilateur.bat](https://raw.githubusercontent.com/MaxiStudioDev/Compilateur-MSL5.0/main/Compilateur/4.5/setup_and_compile.bat)
