---
title = "Home"
---

<article class="media">
    <figure class="media-left">
        <p class="image is-96x96">
            <img src="/static/assets/robert.jpg" alt="Robert Yin">
        </p>
    </figure>
    <div class="media-content">
        <h1 class="title has-text-weight-bold is-size-5-mobile">bob·ert·o</h1>
        <p class="subtitle is-italic is-size-6-mobile">noun</p>
        <p>A portmanteau:
            <a href="https://en.wiktionary.org/wiki/Bobert"><strong>Bobert</strong></a>
            +
            <a href="https://en.wiktionary.org/wiki/Roberto"><strong>Roberto</strong></a>.
        </p>
    </div>
</article>

<article 
    class="message"
    hx-get="/currently_playing"
    hx-trigger="load"
    hx-target="#message-body"
    hx-on::before-request="resetMessage()"
    hx-on::after-request="resetMessageReload()">
    <div class="message-header p-2">
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon"><i class="ph-bold ph-broadcast"></i></span>
            <span>Activity</span>
        </span>
        <span id="reload"
            class="icon"
            hx-get="/currently_playing"
            hx-trigger="click"
            hx-target="#message-body"
            hx-on::before-request="resetMessage()"
            hx-on::after-request="resetMessageReload()">
            <i class="ph-bold ph-arrows-clockwise"></i>
        </span>
    </div>
    <div id="message-body" class="message-body">
        <article class="media">
            <figure class="media-left">
                <p class="image is-64x64 is-skeleton">
                    <img>
                </p>
            </figure>
            <div class="media-content is-skeleton">
                <strong>song</strong> <small>by artist - some time ago</small>
            </div>
        </article>
    </div>
</article>

---

Howdy! I'm Robert Yin, a software engineer who dabbles in making important things and specializes in making silly things.

> To all my past, present, and future employers: I promise that previous statement was just a joke.

I'm not currently looking for job opportunities, but still feel free to reach out!

<ul class="icon-list ml-0">
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon has-text-danger"><i class="ph-bold ph-map-pin"></i></span>
            <span>Madison, WI</span>
        </span>
    </li>
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon has-text-warning"><i class="ph-bold ph-briefcase"></i></span>
            <span>Sofware Developer at <a href="https://epic.com">Epic Systems</a></span>
        </span>
    </li>
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon has-text-link"><i class="ph-bold ph-graduation-cap"></i></span>
            <span>BSCS from <a href="https://northeastern.edu">Northeastern University</a></span>
        </span>
    </li>
</ul>

<ul class="icon-list ml-0">
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon has-text-primary"><i class="ph-bold ph-envelope"></i></span>
            <span><a href="mailto:bobertoyin@gmail.com">bobertoyin@gmail.com</a></span>
        </span>
    </li>
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon has-text-success"><i class="ph-bold ph-read-cv-logo"></i></span>
            <span><a href="/static/assets/resume.pdf">resume.pdf</a></span>
        </span>
    </li>
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon"><i class="ph-bold ph-github-logo"></i></span>
            <span><a href="https://github.com/bobertoyin">@bobertoyin</a></span>
        </span>
    </li>
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon has-text-info"><i class="ph-bold ph-linkedin-logo"></i></span>
            <span><a href="https://linkedin.com/in/boberto">@boberto</a></span>
        </span>
    </li>
</ul>

## (In)Frequently Asked Questions

> Does anyone actually call you Boberto?

Nope. A few folks have used Bobert and Roberto, but I generally go by Robert. Boberto is just for online accounts and usernames.

> Favorite programming language?

Obligatory "there is no best language, just choose the right language for the job" statement here.

```python
def programming() -> str:
    "I enjoy partaking in a little Python and tomfoolery"
```

```rust
fn programming() -> Result<'static str, Box<dyn Error>> {
    Ok("I enjoy partaking in a little Rust and tomfoolery")
}
```

[Emojicode](https://emojicode.org) truthers rise up!

> Any hobbies outside of programming/software?

I dabble in things. Rule of thumb for hobbies: get good, but never good enough to do it for weddings.

<ul class="icon-list ml-0">
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon"><i class="ph-bold ph-chef-hat"></i></span>
            <span>Don't ask me to cater your wedding</span>
        </span>
    </li>
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon"><i class="ph-bold ph-camera"></i></span>
            <span>Don't ask me to photograph your wedding</span>
        </span>
    </li>
    <li>
        <span class="icon-text is-flex-wrap-nowrap">
            <span class="icon"><i class="ph-bold ph-tire"></i></span>
            <span>Don't ask me to kickflip at your wedding</span>
        </span>
    </li>
</ul>
