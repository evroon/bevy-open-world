<h1 align="center">
    Bevy Volumetric Clouds
</h1>

## Credits

The overall rendering approach is inspred by the cloud rendering in the Himalays ([shadertoy](https://www.shadertoy.com/view/MdGfzh)), created by Reinder Nijhoff.
However, the implementation is very different:

1. Rendering happens in a compute shader instead of fragment shader
2. Shaders are written in WGSL instead of GLSL
3. A lot of refactoring

Resources:

1. "The real-time volumetric cloudscapes of Horizon Zero Dawn" by Andrew Schneider and Nathan Vos ([article](https://www.guerrilla-games.com/read/the-real-time-volumetric-cloudscapes-of-horizon-zero-dawn))
2. "Physically Based Sky, Atmosphere and Cloud Rendering in Frostbite" by Sébastien Hillaire ([pdf](https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/s2016-pbs-frostbite-sky-clouds-new.pdf))
3. Himalays: created by Reinder Nijhoff 2018 ([shadertoy](https://www.shadertoy.com/view/MdGfzh))
4. Temporal reprojection: "Rain Forest" (by Íñigo Quílez) ([article](https://www.shadertoy.com/view/4ttSWf))

## License

Licensed under [MIT](https://choosealicense.com/licenses/mit/), see [LICENSE](../../LICENSE).
