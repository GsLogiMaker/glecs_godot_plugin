# Glecs - Godot (fast) Lightweight Entity Component System
This plugin is a wrapper around Flecs to add an entity component system to Godot.
## Branches
### Main
The `main` branch is kept stable and ready to install and use within a project's `asset` folder. To install in your project run the following in your Godot project's `addons` folder:
```
git clone https://github.com/GsLogiMaker/glecs_godot_plugin.git
```
### Dev
The `dev` branch is where main development happens. This branch contains a Godot project with the addon source code, unit tests, and developer tools. To install for development run the following command:
```
git clone https://github.com/GsLogiMaker/glecs_godot_plugin.git -b dev
cd glecs_godot_plugin
git submodule update --init --recursive
```
### Publish
The `publish` branch is properly formatted to be used in the asset store.
