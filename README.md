# Git Server Constructs

<!-- Badges? -->

A collection of constructs for [tf-bindgen] used to deploy a Git server based on
[Gitea] to Kubernetes.

[tf-bindgen]: https://github.com/robert-oleynik/tf-bindgen
[Gitea]: https://gitea.io/en-us/

## Requirements

- [Cargo](https://doc.rust-lang.org/cargo/)
- [Terraform](https://www.terraform.io/)

## Usage

This project will expose a CLI application used to run and configure the deployment.
Run the following command to get some help:

```sh
cargo run -- --help
```

You can use the provided binary to deploy/provision the git server.

## Configuration

This binary use the file `gitserver.toml` to specify deployment specific information.
The configuration will use the following fields:

```toml
[server]
domain = "<domain or IP>" # Domain/IP required to setup correct routing.
node = "<node name>" # Kubernetes node to link this deployment to.

# Gitea root user configuration
[root]
user = "root" # Root user name
passwd = "..." # Root user password
email = "root@localhost" # E-Mail of root user
```

## Components

This repository contains infrastructure as code to deploy a git server with CI:

- [Gitea](https://gitea.io/) as git server
	- [Postgresql](https://www.postgresql.org/) as database for gitea.
	- [Memcached](https://www.memcached.org/) as cache for gitea.
- [Jenkins](https://www.jenkins.io/) as CI server

## Roadmap

<!-- Upcoming changes -->

## Contributing

<!-- TODO: add placeholder text -->

## License

This project is licensed under the [BSD-3-Clause](./LICENSE) license.
