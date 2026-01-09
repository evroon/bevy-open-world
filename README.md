<h1 align="center">
    Bevy Open World
</h1>

<p align="center">
  <a href="https://github.com/evroon/adsb-globe/actions"
    ><img
      src="https://img.shields.io/github/actions/workflow/status/evroon/adsb-globe/ci.yml"
      alt="build status"
  /></a>
</p>

![adsb](.github/screenshots/adsb.png)

This repo contains various crates for open world rendering in Bevy

Currently includes:

- [bevy-skybox](/crates/bevy-skybox/): A plugin to render volumetric clouds
- [bevy-fly-camera](/crates/bevy-fly-camera/): A plugin for a camera flying above terrain surface
- [bevy-where-was-i](/crates/bevy-where-was-i/): A utility plugin to save transforms to disk

## Resources

- ADS-B data: [ADSB.lol](https://github.com/adsblol/globe_history_2025)
- Earth textures: NASA Blue Marble Next Generation, downloaded from [Solar System Scope](https://www.solarsystemscope.com/textures/). License: CC BY 4.0
- Milky way night sky: [streets.gl](https://github.com/StrandedKitty/streets-gl/tree/dev/src/resources/textures/starmap). License: MIT
- Clickhouse: [adsb.exposed](https://github.com/ClickHouse/adsb.exposed/) for inspiration on how to store the ads-b data. License: CC BY-NC-SA 4.0
