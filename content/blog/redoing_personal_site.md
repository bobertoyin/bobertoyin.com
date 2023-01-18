+++
title = "Redoing My Personal Site"
date = 2022-12-23
updated = 2023-01-18
[taxonomies]
topics = ["web development", "personal project"]
+++

> If at first you don't succeed, pretend you don't know where your old site went.<!-- more -->

After getting my own domain name, I figured I should upgrade my old, somewhat crusty personal site into a sleeker one. 
Of course, instead of preserving the old site's code for a proper compare-and-contrast, or even a retrospective, I decided to wipe my git history in favor of historical revisionism. Yay! In light of this ~~massive~~ minor error, I'll go over some of the decisions that I made when planning this new site.

## Tooling :hammer_and_wrench:

### Static Site Generators :memo:

Although it was fun to do in high school, building a personal site from plain HTML and CSS files does not appeal to me at all. Instead, what I needed was a **static site generator**, which would allow me to create new content without having to worry about excessive styling and markup. Thus, I sat down to consider a few of the popular options available:

* **Jekyll**: I've used it before, but last time I used it I had some issues with my Ruby environment.
* **Hugo**: It felt too complex and unwieldy for my use case (or perhaps just too cool for my mushy brain).
* **Zola**: It seemed simple enough for me and built in Rust (I'm willing to buy into the Rust hype like a goober).

Zola was in fact simple enough for me to scaffold my site pages and general stylings, although nicer features like topic tagging and post pagination were complex enough to put them on the backburner at first.

### Deployment :rocket:

Unfortunately, the Zola site proved to be difficult to deploy on my initial platforms of choice: GitHub's [Zola Deploy](https://github.com/marketplace/actions/zola-deploy) Action would only render my README as the GitHub Page instead of the actual site, and DigitalOcean's [App Platform](https://www.digitalocean.com/products/app-platform) didn't have Zola support. In the end, I settled on deploying to [Netlify](https://netlify.com), as it gave me the least issues with Zola builds and deploys. Frankly, I'm not sure why this was the case.

## Design :triangular_ruler:

### CSS Framework :crayon:

Given that I haven't really improved by webpage design skills since high school, I wanted to leverage a Sass framework that would play well with Zola. I ended up settling on [Bulma](https://bulma.io) for most of its pre-defined stylings and responsive layouts. I also decided to have Bulma installed via [npm](https://www.npmjs.com) to customize some of Bulma's variables without having a static copy sitting around in the code repository. I'm happy with the results so far, especially since I don't have to deal with the responsiveness of the site myself.

### Fonts :abcd:

The following fonts are pre-loaded via Google Fonts: [Public Sans](https://public-sans.digital.gov) and [Source Code Pro](https://github.com/adobe-fonts/source-code-pro). There wasn't any particular reason for choosing these two fonts, other than ease of access and minimalist style.


### Color Theme :art:

I ended up settling on [OneHalf](https://github.com/sonph/onehalf)'s light theme after using the Gruvbox's dark theme for so long. As much as I enjoy Gruvbox, I wanted less of a "retro" look, as retro design cues have never really been a hallmark of mine. I also really like OneHalf's primary colors, as they add a nice pop to the site without being harsh on the eyes.
