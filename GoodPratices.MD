# Liste des bonnes pratiques à appliquer sur le projet pour la partie Backend API

*Note: C'est pratiques peuvent évoluer en fonction des retours d'expériences de l'équipe, n'hésitez pas a remonter vos suggestions.*

## Architecture

### Dossier

- **api**: Contient les fichiers de configuration de l'APIs
- **database**: Contient les fichiers de configuration de la base de données
- **models**: Contient les modèles de données
- **services**: Contient les services de l'application
- **utils**: Contient les utilitaires de l'application

## Tests

### Composition des tests

- Les tests sont écrits en utilisant le format suivant: `fn test_name()`
- Les tests sont écrits en utilisant le framework de test de Rust (https://doc.rust-lang.org/book/ch11-01-writing-tests.html)

## Documentation

### Composition de la documentation

- La documentation est écrite en Markdown
- La documentation est écrite en anglais
- La documentation est écrite en utilisant le format Markdown de Github (https://guides.github.com/features/mastering-markdown/)

### Composition de la documentation du code

- La documentation du code est écrite en utilisant le format Rust (https://doc.rust-lang.org/reference/comments.html)
- La documentation du code est écrite en anglais
- **La documentation du code est obligatoire avant de faire une Pull Request**

## Conventions de code

### Composition des noms de variables

- Les noms de variables sont écrits en snake_case
- Les noms de variables sont explicites
- Les noms de variables sont en anglais

### Composition des noms de fonctions

- Les noms de fonctions sont écrits en snake_case
- Les noms de fonctions sont explicites
- Les noms de fonctions sont en anglais

## Conventions de commit et pull request

- Les messages de commit sont écrits en anglais
- Les messages de commit sont écrits en utilisant le format suivant: `type(scope): subject`

- Les pull requests sont écrites en français
- Les pull requests sont écrites en utilisant le format suivant: `type(scope): subject`
- Une pull request ne doit pas contenir plus de 500 lignes de code
- Une pull request doit contenir l'avancée du travail effectué et les points bloquants
- Une pull request doit contenir des captures d'écran de la fonctionnalité développée

### Types de commit (type) et pull request (type)

- **feat**: Ajout d'une nouvelle fonctionnalité
- **fix**: Correction d'un bug
- **docs**: Modification de la documentation
- **refactor**: Modification du code sans ajouter de fonctionnalité ou corriger un bug
- **perf**: Modification du code pour améliorer les performances
- **test**: Ajout ou modification de tests
- **build**: Modification de la configuration de build
- **ci**: Modification de la configuration de CI
- **chore**: Modification de la configuration de l'application
- **revert**: Revert d'un commit précédent

### Scopes de commit (scope) et pull request (scope)

- **database**: Modification de la configuration de la base de données
- **models**: Modification d'un modèle de données
- **services**: Modification d'un service
- **utils**: Modification d'un utilitaire
- **tests**: Modification d'un test
- **docs**: Modification de la documentation
- **config**: Modification de la configuration de l'application
- **build**: Modification de la configuration de build
- **ci**: Modification de la configuration de CI
- **deps**: Modification des dépendances
- **other**: Autre modification
