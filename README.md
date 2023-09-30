# Godot Entity Component System Plugin (gECS)
This plugin is a wrapper around flecs to add an entity component system to Godot.

## Branches
### Main
The `main` branch is kept stable and ready to install and use within a project's `asset` folder. To install in your project run the following in your Godot project's `addons` folder:
```
git install https://github.com/GsLogiMaker/g_ecs_plugin.git
```

### Publish
The `publish` branch is properly formatted to be used in the asset store.

### Dev
The `dev` branch is where main development happens and contains extra source code for the compiled binaries. To install for development run the following command:
```
git install https://github.com/GsLogiMaker/g_ecs_plugin.git
git switch dev
git submodule update --init --recursive
```
