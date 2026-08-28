<img src="assets/logo.svg" alt="Homeostat" width="180" align="left">

<h3>Homeostat</h3>

<p>A learning-augmented control plane for sharded systems.</p>

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/status-experimental-blueviolet?style=flat-square)](https://github.com/homeostat-ai/homeostat)
[![Stars](https://img.shields.io/github/stars/homeostat-ai/homeostat?style=flat-square&logo=github)](https://github.com/homeostat-ai/homeostat)

<br clear="left">

## Get started

Download the latest Homeostat release for your platform from
[GitHub Releases](https://github.com/homeostat-ai/homeostat/releases/latest),
extract it, then start the controller:

```console
./homeostat controller start
```

Check that it is running:

```console
curl http://localhost:9090/healthz
```
