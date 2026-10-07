# Compilateur MSL5.0

Un compilateur et un environnement de simulation pour le langage MSL5.0, écrit en Rust. Le projet permet de transformer des programmes MSL5 en instructions binaires exploitable par un simulateur ou une autre application.

## Vue d'ensemble

MSL5.0 est un langage de programmation orienté logique/contrôle, avec des notions de registres, de conditions, de boucles et de GPIO. Ce dépôt contient :

- un compilateur Rust qui parse et transforme le code MSL5.0 en instructions encodées,
- un simulateur Windows pour visualiser le comportement du programme,
- un plugin VS Code pour faciliter l'écriture du langage,
- des exemples de programmes et des sorties compilées.

## Fonctionnalités

- Analyse syntaxique de scripts MSL5.0
- Support des variables et registres
- Gestion des conditions `IF`, `WHILE`, `LOOP`, `BREAK`
- Support des opérations ALU (`ADD`, `SUB`, `AND`, `OR`, `XOR`, etc.)
- Gestion de la logique d'entrée/sortie GPIO (`IN`, `OUT`, `IF IN`)
- Génération de fichiers binaire (`.bin`) et texte (`.txt`)
- Extension VS Code avec coloration syntaxique et snippets
- Architecture modulaire selon les différents composants du projet

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
│       ├── Simulator/            # Simulateur Windows
│       │   ├── src/
│       │   └── Cargo.toml
│       ├── Spliter/              # Outil de découpage/splitting
│       ├── PluginAU/             # Extension VS Code
│       └── msl5-mini-pro-4.5.0.vsix
└── ...
```

## ⚠️ IMPORTANT - Structure du dossier Compilateur

**Le dossier `Compilateur` et toute son arborescence DOIVENT rester intacts dans le repository.**

**Ne pas modifier, déplacer ou supprimer :**
- ✅ La structure complète du dossier `Compilateur/4.5/`
- ✅ Les fichiers de sortie du compilateur : `OpC`, `D1`, `D2`
- ✅ L'emplacement exact du compilateur Rust dans `Compilateur/4.5/compilator/`

**Pourquoi ?** L'émulateur CPU 32-Bits (Cpu-32-Bits-Project-MSL5P) dépend de ces fichiers aux emplacements spécifiques. Si vous modifiez l'arborescence ou déplacez les fichiers, l'émulateur ne pourra pas les localiser et cessera de fonctionner correctement.

**Recommandation** : Copiez le compilateur Rust vers votre projet, mais ne touchez pas à la structure originale du dépôt.

## Prérequis

- Rust (dernière version stable recommandée)
- Cargo
- Pour le simulateur : Windows
- Pour l'extension VS Code : VS Code ou une base compatible

## Compilation du compilateur

Depuis le dossier `Compilateur/4.5/compilator` :

```bash
cargo build
```

Pour compiler un fichier source MSL5 :

```bash
cargo run -- path/to/file.msl5 output.bin
```

Exemple :

```bash
cargo run -- test.msl5 out.bin
```

Le compilateur générera :

- `out.bin` : version binaire du programme
- `out.txt` : représentation textuelle des instructions
- `OpC`, `D1`, `D2` : fichiers de sortie associés (** NE PAS MODIFIER LEUR EMPLACEMENT **)

## Exemple de programme

```text
let x = 10
let y = 20

if x == y {
    out(1).set(1)
} else {
    out(1).set(0)
}
```

Le langage supporte un style de programmation proche des instructions logiques temporisées et de contrôle d'entrées/sorties.

## Simulation

Le simulateur se trouve dans :

```text
Compilateur/4.5/Simulator
```

Il est conçu pour une utilisation sous Windows et sert à tester les programmes MSL5.0 dans une interface graphique.

## Extension VS Code

L'extension est située dans :

```text
Compilateur/4.5/PluginAU
```

Elle apporte :

- coloration syntaxique pour MSL5.0,
- snippets de code,
- configuration de langage dans VS Code.

Pour installer l'extension locale :

```bash
code --install-extension msl5-mini-pro-4.5.0.vsix
```

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
