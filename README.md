# Personal Site

[![Netlify Status](https://api.netlify.com/api/v1/badges/d5757a57-49b9-4b93-bd62-f12eb1a88f46/deploy-status)](https://app.netlify.com/sites/bobertoyin/deploys)

The home of all my ~~terrible~~ awesome ideas.

## Develop

To get started, install [Zola](https://www.getzola.org) and [Node.js + npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm).

```shell
# install dependencies
npm install

# serve site locally
zola serve

# build locally
zola build
```

## Versioning System

See the `version` key in the `config.toml` for the current version in the form of `MAJOR.MINOR.PATCH`. Versioning is done as a cursed spin-off of semantic versioning and other similar systems: major design updates warrant a `MAJOR` bump, new content or minor features warrant a `MINOR` bump, and bug fixes warrant a `PATCH` bump. 