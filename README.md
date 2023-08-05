[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/MinecraftWebhook-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/MinecraftWebhook-rust)


# `MinecraftWebhook`
Welcome to `MinecraftWebhook` 🎉

This crate provides HTTP webhooks to trigger predefined minecraft commands.


## Example config
```toml
[server]
address = "127.0.0.1:8080"

[rcon]
address = "example.org:25575"
password = "insertsupersecurepasswordhere"

[webhooks]
hello-world = "say Hello World"
seed = "seed"
```
