---
title = "Change Log"
---

# Change Log

---

## `v3.2.2`

- Fixed Docker build issues stemming from missing folder copying and incorrectly naming the built binary

## `v3.2.1`

- Improved Docker build times, build caching opportunities, and image size
  - Accomplished with [`cargo-chef`](https://github.com/LukeMathWalker/cargo-chef) and [distroless images](https://github.com/GoogleContainerTools/distroless)
- Fixed issue with the sun icon in the theme-toggle button briefly flashing when using the light color theme
- Simplified and added new [`Error`](https://doc.rust-lang.org/std/error/index.html) types within the server code
- Removed the requirement of a Github personal access token to run the site

## `v3.2.0`

- Added content to the `projects` page
- Used CDN sources for [Phosphor Icons](https://phosphoricons.com) and [htmx](https://htmx.org)
- Made footer stick to the bottom of the browser when page content is not scrollable

## `v3.1.1`

- Added missing `openssl-dev` package to `Dockerfile`

## `v3.1.0`

- Added currently/last listened to song via [Last.fm client](https://docs.rs/lastfm/latest/lastfm/index.html)
- Added header divider to `changelog` page

## `v3.0.0`

- Migrated from a static site generator ([Zola](https://www.getzola.org)) to a web server framework ([Axum](https://crates.io/crates/axum) + [Tera](https://keats.github.io/tera))
- Upgraded to [Bulma v1](https://bulma.io)
  - Enabled dark theme + theme toggling support!
- Migrated icons from [Font Awesome Icons](https://fontawesome.com) to [Phosphor Icons](https://phosphoricons.com)
- Replaced [Public Sans](https://fonts.google.com/specimen/Public+Sans) and [Source Code Pro](https://fonts.google.com/specimen/Source+Code+Pro) fonts with [Berkeley Mono](https://berkeleygraphics.com/typefaces/berkeley-mono)
- Dropped reference to `CC BY NC SA 4.0` license
- Added a `projects` page
- Spruced up general styling inconsistencies/oddities
- Updated information on the home page
