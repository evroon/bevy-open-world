<h1 align="center">
    Bevy, where was I?
</h1>

<p align="center">
  <a href="https://crates.io/crates/bevy_where_was_i"
    ><img
      src="https://img.shields.io/crates/v/bevy_where_was_i"
      alt="crate on crates.io"
  /></a>
  <a href="https://docs.rs/bevy_where_was_i"
    ><img
      src="https://img.shields.io/docsrs/bevy-where-was-i"
      alt="docs on docs.rs"
  /></a>
  <a href="https://github.com/evroon/bevy-where-was-i/actions"
    ><img
      src="https://img.shields.io/github/actions/workflow/status/evroon/bevy-where-was-i/ci.yml"
      alt="build status"
  /></a>
</p>

A tiny Bevy library that saves the camera position when the program closes, and restores it when
you launch the application again. This is useful when debugging and you don't want to continuously
change your hardcoded camera coordinates.

Note that this actually works for any `Transform` component, and not only restores translation
information, but also scale and rotation.

This crate is compatible with [bevy_panorbit_camera](https://github.com/Plonq/bevy_panorbit_camera).

## Usage

Add the plugin:

```rust ignore
.add_plugins(WhereWasIPlugin::default())
```

### For a camera

To save the translation and rotation of a camera, add the `WhereWasI` component to an entity with a
`Camera` component:

```rust ignore
commands.spawn((
    Camera::default(),
    WhereWasI::camera(),
));
```

Note: `WhereWasI::camera()` is equivalent to `WhereWasI::from_name("camera")`.

### For other entities

For other entities, a name has to be provided.
Add the `WhereWasI` component to an entity with a `Transform`:

```rust ignore
commands.spawn((
    PointLight::default(),
    WhereWasI::from_name("point_light"),
));
```

Since `WhereWasI` indicates that `Transform` is a required component, we can omit it and
`WhereWasI` will construct it. If you want to change the initial state of the `Transform` before a
savefile exists, add a `Transform` component to the bundle:

```rust ignore
commands.spawn((
    PointLight::default(),
    Transform::from_xyz(5.0, 2.0, 5.0),
    WhereWasI::from_name("point_light"),
));
```

See the
[3D scene example](https://github.com/evroon/bevy-where-was-i/blob/master/examples/3d_scene.rs)
for a complete example.

### Save files

The save files will by default be stored in `./assets/saves`. You likely want to add this directory
to you `.gitignore`. Alternatvely, you can configure a different directory when initializing
the plugin. For example, you can store the savefiles in the user's `.config` directory:

```rust ignore
.add_plugins(WhereWasIPlugin {
    directory: "~/.config/bevy-saves/my-game".into(),
})
```

`WhereWasIPlugin` will make sure the directory exists if it doesn't already.

## Bevy version compatibility

| bevy | bevy-where-was-i     |
|------|----------------------|
| 0.17 | 0.3.*                |
| 0.16 | 0.2.*                |
| 0.15 | 0.1.*                |

## License

Licensed under [MIT](https://choosealicense.com/licenses/mit/), see [LICENSE](../../LICENSE).
