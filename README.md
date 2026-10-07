# Compilateur MSL5.0

Un compilateur et un environnement de simulation pour le langage MSL5.0, écrit en Rust. Le projet permet de transformer des programmes MSL5 en instructions binaires exploitable par un émulateur CPU 32-Bits via **AntaresCircuit.io**.

## Vue d'ensemble

MSL5.0 est un langage de programmation orienté logique/contrôle, avec des notions de registres, de conditions, de boucles et de GPIO. Ce dépôt contient :

- un compilateur Rust qui parse et transforme le code MSL5.0 en instructions encodées,
- un plugin VS Code pour faciliter l'écriture du langage,
- des exemples de programmes et des sorties compilées,
- une intégration complète avec l'émulateur CPU 32-Bits (AntaresCircuit.io).

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
│       └── msl5-mini-pro-4.5.0.vsix
└── ...
```

## Guide de démarrage rapide

Suivez ces étapes dans l'ordre pour configurer correctement votre environnement :

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

### Étape 3 : Configurer le compilateur Rust

#### Prérequis

- Rust (dernière version stable recommandée)
- Cargo
- Git (optionnel, pour cloner le repository)

#### Installation et placement du dossier Compilateur

1. **Clonez ou téléchargez** ce repository :
   ```bash
   git clone https://github.com/MaxiStudioDev/Compilateur-MSL5.0.git
   ```

2. **Copiez le dossier `Compilateur`** complet depuis ce repository

3. **Collez-le dans votre dossier Documents** :
   ```
   C:\Users\[VotreNomUtilisateur]\Documents\Compilateur\
   ```

4. **Vérifiez que l'arborescence est intacte** :
   - ✅ `Documents/Compilateur/4.5/compilator/`
   - ✅ `Documents/Compilateur/4.5/PluginAU/`
   - ✅ Les fichiers `OpC`, `D1`, `D2` doivent rester intacts

#### Compilation du compilateur Rust

Depuis le terminal (PowerShell, CMD ou Git Bash), naviguez vers le dossier compilateur :

```bash
cd Documents/Compilateur/4.5/compilator
cargo build
```

Si vous voulez tester immédiatement :

```bash
cargo run -- test.msl5 out.bin
```

### Étape 4 : Installation optionnelle du plugin VS Code

Le plugin VS Code facilite l'écriture de programmes MSL5.0 avec coloration syntaxique et snippets.

1. Ouvrez VS Code

2. Allez dans **Extensions** (Ctrl+Shift+X ou Cmd+Shift+X)

3. Installez l'extension locale `.vsix` :
   ```bash
   code --install-extension Documents/Compilateur/4.5/msl5-mini-pro-4.5.0.vsix
   ```

   Ou installez-la manuellement :
   - Allez à **Extensions** → **...** (en haut à droite) → **Install from VSIX...**
   - Naviguez vers `Documents/Compilateur/4.5/msl5-mini-pro-4.5.0.vsix`
   - Confirmez

### Étape 5 : Créer et compiler votre premier programme

1. **Créez un fichier** `mon_programme.msl5` dans un dossier au choix :
   ```msl5
   let x = 10
   let y = 20

   if x == y {
       out(1).set(1)
   } else {
       out(1).set(0)
   }
   ```

2. **Compilez le programme** depuis le terminal :
   ```bash
   cd Documents/Compilateur/4.5/compilator
   cargo run -- /path/to/mon_programme.msl5 out.bin
   ```

3. **Les fichiers générés** :
   - `out.bin` : version binaire du programme
   - `out.txt` : représentation textuelle des instructions
   - `OpC`, `D1`, `D2` : fichiers de sortie pour l'émulateur

4. **Chargez le résultat dans AntaresCircuit.io** :
   - Ouvrez AntaresCircuit.io
   - Chargez le fichier `out.bin` généré
   - Lancez la simulation

## ⚠️ IMPORTANT - Emplacement obligatoire du dossier Compilateur

**Le dossier `Compilateur` DOIT être obligatoirement situé dans le dossier `Documents` de votre ordinateur.**

**Chemin requis :** `C:\Users\[VotreNomUtilisateur]\Documents\Compilateur\`

**Pourquoi ?** L'émulateur AntaresCircuit.io et le projet Cpu-32-Bits-Project-MSL5P recherchent les fichiers de sortie (`OpC`, `D1`, `D2`) et le compilateur à cet emplacement exact. Si vous le placez ailleurs, l'émulateur ne trouvera pas les fichiers et cessera de fonctionner.

## ⚠️ IMPORTANT - Ne pas modifier l'arborescence du dossier Compilateur

**Le dossier `Compilateur` et toute son arborescence DOIVENT rester intacts après l'installation initiale.**

**Ne pas modifier, déplacer ou supprimer :**
- ✅ La structure complète du dossier `Compilateur/4.5/`
- ✅ Les fichiers de sortie du compilateur : `OpC`, `D1`, `D2`
- ✅ L'emplacement exact du compilateur Rust dans `Compilateur/4.5/compilator/`
- ✅ Tous les fichiers de configuration et de dépendances

**Pourquoi ?** L'émulateur AntaresCircuit.io (Cpu-32-Bits-Project-MSL5P) dépend de ces fichiers aux emplacements spécifiques. Si vous modifiez l'arborescence ou déplacez les fichiers, l'émulateur ne pourra pas les localiser et cessera de fonctionner correctement.

**Recommandation** : Une fois le dossier correctement placé dans Documents, ne le touchez plus. Copiez les fichiers dont vous avez besoin dans d'autres projets, mais préservez l'intégrité du dossier `Compilateur/` dans Documents.

## Compilation d'un fichier MSL5

Depuis le dossier `Documents/Compilateur/4.5/compilator` :

```bash
cargo run -- /chemin/complet/vers/fichier.msl5 nom_sortie.bin
```

Exemple :

```bash
cargo run -- C:/Users/MonUtilisateur/Desktop/test.msl5 out.bin
```

Le compilateur générera :

- `out.bin` : version binaire du programme
- `out.txt` : représentation textuelle des instructions
- `OpC`, `D1`, `D2` : fichiers de sortie associés (** NE PAS MODIFIER LEUR EMPLACEMENT **)

## Exemple de programme MSL5.0

```msl5
let x = 10
let y = 20

if x == y {
    out(1).set(1)
} else {
    out(1).set(0)
}
```

Le langage supporte un style de programmation proche des instructions logiques temporisées et de contrôle d'entrées/sorties.

## Extension VS Code

L'extension VS Code pour MSL5.0 est située dans :

```
Documents/Compilateur/4.5/PluginAU
```

Elle apporte :

- ✅ Coloration syntaxique pour MSL5.0
- ✅ Snippets de code
- ✅ Configuration de langage dans VS Code
- ✅ Auto-complétion basique

Pour installer l'extension locale :

```bash
code --install-extension Documents/Compilateur/4.5/msl5-mini-pro-4.5.0.vsix
```

Après installation, redémarrez VS Code. Les fichiers `.msl5` seront automatiquement colorisés.

## Workflow complet : de la programmation à la simulation

1. **Écrivez votre code** dans VS Code (avec le plugin)
2. **Compilez** via terminal : `cargo run -- mon_code.msl5 out.bin`
3. **Vérifiez les sorties** : `out.bin`, `out.txt`, `OpC`, `D1`, `D2`
4. **Ouvrez AntaresCircuit.io** avec le CPU 32-Bits (CPUv5.0.acp) importé
5. **Chargez** le fichier `out.bin` généré
6. **Lancez la simulation** pour visualiser le comportement de votre programme

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

Liens utiles :
- 🔗 [AntaresCircuit.io](https://antarescircuit.io)
- 🔗 [Cpu-32-Bits-Project-MSL5P](https://github.com/MaxiStudioDev/Cpu-32-Bits-Project-MSL5P)
- 🔗 [Repository Compilateur-MSL5.0](https://github.com/MaxiStudioDev/Compilateur-MSL5.0)
