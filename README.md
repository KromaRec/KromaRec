<div align="center">

<img src="assets/kromarec-logo.png" alt="KromaRec Logo" width="180">

<h1>KromaRec</h1>

<p>
  Open-source server software for Rec Room
</p>

<p>
  A lightweight, self-hosted backend alternative focused on
  performance, compatibility, experimentation, and community-driven development.
</p>

<p>
  <img src="https://img.shields.io/badge/Rust-Backend-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/License-Apache%202.0-blue" alt="Apache 2.0">
  <img src="https://img.shields.io/badge/Platform-Linux-lightgrey?logo=linux" alt="Linux">
  <img src="https://img.shields.io/badge/Status-Development-yellow" alt="Development">
</p>

</div>

<hr>

<h3 align="center">About KromaRec</h3>

<p align="center">
  KromaRec is an experimental, open-source backend implementation designed
  to provide a self-hosted server environment for compatible Rec Room clients.
  Built primarily in Rust, it focuses on low resource consumption,
  efficient networking, and independent hosting.
</p>

<h3 align="center">Features</h3>

<p align="center">
  Lightweight Rust backend · SQLite database · Authentication and sessions
  <br>
  Rooms and matchmaking · Photon networking integration · Real-time notifications
  <br>
  Alpine Linux support · Static musl builds · Self-hosted infrastructure
</p>

<h3 align="center">Performance</h3>

<p align="center">
  KromaRec is designed to minimize memory usage and runtime overhead
  without sacrificing essential backend functionality.
</p>

<div align="center">

| Benchmark | Memory Usage |
|:---|---:|
| Backend idle | ~3.4 MB |
| Backend after HTTP activity | ~6.1 MB |
| Backend + Luxon (2 synthetic peers) | ~7.6 MB |

<sub>
  Measurements from KromaRec 0.10.1 test environments.
  Actual resource usage may vary depending on workload and connected players.
</sub>

</div>

<h3 align="center">Development Status</h3>

<p align="center">
  KromaRec is currently under active development.
  Automated backend and networking tests have been performed,
  but full compatibility with native Rec Room clients
  and playable multiplayer sessions has not yet been verified.
</p>

<hr>

<p align="center">
  <sub>
    <b>Credits:</b> KromaRec acknowledges
    <a href="https://github.com/recflare">RecFlare</a>
    for its work on Rec Room backend compatibility and as a reference
    during the development of this project.
    KromaRec is an independent project and is not officially affiliated with RecFlare.
  </sub>
</p>

<h3 align="center">Links</h3>

<p align="center">
  <a href="https://github.com/KromaRec/KromaRec">Server Repository</a> ·
  <a href="https://github.com/KromaRec/KromaRecClient">Client Repository</a> ·
  <a href="https://github.com/KromaRec/KromaNet">KromaRec's Website</a> ·
  <a href="https://github.com/KromaRec/KromaNetBot">KromaNet Discord Bot</a> ·
  <a href="https://discord.gg/qwXY9QG6qm">Discord</a>
</p>
