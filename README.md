# Grafana Backend Plugin Sample in Rust

This is a sample plugin for building Grafana Backend Plugins using the Rust SDK.

## What is Grafana Backend Plugin?

Grafana Backend Plugins are plugins that comprise both a frontend _and_ backend component. They can provide datasource implementations in cases where the datasource cannot be communicated with by the browser using HTTP, or can provide entire app experiences inside Grafana, or even the two combined.

This sample app contains a backend datasource plugin, but the backend component also implements a few other backend services just to serve as an example.

For more information about backend plugins, refer to the documentation on [Backend plugins](https://grafana.com/docs/grafana/latest/developers/plugins/backend/).

## Getting started

### Docker-compose

The simplest way to get started is with the included docker-compose file. Simply run

```bash
docker-compose up
```

which will start three services:

1. A Grafana instance set up for development mode, serving on port 3000
2. A frontend build service, which watches for changes and builds the frontend
3. A backend build service, which watches for changes, builds the backend, moves the binary into the correct location, and signals the current backend component to restart.

Note that if you are on a Mac, the build services may run slowly as they have to use emulation.

A backend plugin consists of both frontend and backend components.

### Frontend

1. Install dependencies

   ```bash
   yarn install
   ```

2. Build plugin in development mode or run in watch mode

   ```bash
   yarn dev
   ```

   or

   ```bash
   yarn watch
   ```

3. Build plugin in production mode

   ```bash
   yarn build
   ```

### Backend

1. Ensure you have a recent version of Rust installed. [rustup](https://rustup.rs/) is the recommended installation method.

   ```bash
   rustup update
   ```

2. Build the backend plugin in debug mode, then copy it to the `dist` directory with the correct name so that Grafana picks it up.

   ```bash
   cd backend
   export GOARCH=darwin_arm64  # replace with your GOARCH
   cargo build
   cp target/debug/grafana-sample-backend-plugin-rust ../dist/gpx_grafana-sample-backend-plugin-rust_${GOARCH}

   # or, using cargo-watch

   cd backend
   export GOARCH=darwin_arm64  # replace with your GOARCH
   cargo watch --why -x build -s 'rm ../dist/gpx_grafana-sample-backend-plugin-rust_${GOARCH} && cp target/debug/grafana-sample-backend-plugin-rust ../dist/gpx_grafana-sample-backend-plugin-rust_${GOARCH}' -c -w . 
   ```

## Learn more

- [Build a data source backend plugin tutorial](https://grafana.com/tutorials/build-a-data-source-backend-plugin)
- [Grafana documentation](https://grafana.com/docs/)
- [Grafana Tutorials](https://grafana.com/tutorials/) - Grafana Tutorials are step-by-step guides that help you make the most of Grafana
- [Grafana UI Library](https://developers.grafana.com/ui) - UI components to help you build interfaces using Grafana Design System
- [Grafana plugin SDK for Rust](https://github.com/sd2k/grafana-plugin-sdk-rust)
