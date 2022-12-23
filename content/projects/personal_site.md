+++
title = "Personal Site"
date = 2022-12-22
description = "Where all sanity goes to die."
+++

After getting my own domain name, I figured I should put a spiffy new personal site there and work a little bit on some site design. Fortunately for me, there are tons of options for scaffolding, implementing, and deploying a static website. Unfortunately for me, I can be a shoddy decision maker at times. This page details those choices, regardless of whether they're rational or irrational.

## Tooling :hammer_and_wrench:

### Static Site Generators :memo:

Although it was fun to do in high school, building a personal site from plain HTML and CSS files does not appeal to me at all. Instead, what I needed was a **static site generator**, which would allow me to create new content without having to worry about excessive styling and markup. Thus, I sat down to consider a few of the popular options available:

* **Jekyll**
    * I've used it before, but last time I used it I had some issues with my Ruby environment.
* **Hugo**
    * It ended up being to complex and unwieldy for my use case (or perhaps just too cool for my mushy brain).
* **Zola**
    * It's simple enough and built in Rust (I'm willing to buy into the Rust hype like a goober).

In the end, I settled on using [Zola](https://www.getzola.org) for my site.

### Deployment :rocket:

Coming soon!

## Design :triangular_ruler:

### CSS Framework :crayon:

Given that I haven't really improved by webpage design skills since high school, I wanted to leverage a Sass framework that would play well with Zola. I ended up settling on [Bulma](https://bulma.io) for most of its pre-defined stylings and responsive layouts. I also decided to have Bulma installed via [npm](https://www.npmjs.com) to avoid having a static copy sitting around in the code repository.

### Fonts :abcd:

The following fonts are pre-loaded via Google Fonts: [Public Sans](https://public-sans.digital.gov) and [Source Code Pro](https://github.com/adobe-fonts/source-code-pro). There wasn't any particular reason for choosing these two fonts, other than ease of access and minimalist style.


### Color Theme :art:

I ended up settling on [OneHalf](https://github.com/sonph/onehalf)'s light theme after using the Gruvbox's dark theme for so long. As much as I enjoy Gruvbox, I wanted less of a "retro" look, as retro design cues have never really been a hallmark of mine. I also really like OneHalf's primary colors, as they add a nice pop to the site without being harsh on the eyes.