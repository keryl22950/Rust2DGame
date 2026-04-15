# Rustopia

Base de projet pour un jeu 2D side-scroller en Rust avec Bevy et WGSL.

## Objectifs

- Vue de côté 2D
- Mouvement de joueur
- Combinaisons magiques avec E/F/G/H
- Sort déclenché avec `Space`
- Shader WGSL pour l'effet visuel du joueur

## Structure

- `Cargo.toml` : dépendances
- `src/main.rs` : logique de base du jeu
- `assets/shaders/magic.wgsl` : shader personnalisé

## Lancer le jeu

1. Installer Rust si ce n'est pas déjà fait
2. Exécuter :

```powershell
cargo run
```

## Améliorations possibles

- remplacer le joueur par un sprite 2D
- ajouter des ennemis et des plateformes
- ajouter plusieurs sorts avec effets visuels
- créer un système de particules et de stamina magique
