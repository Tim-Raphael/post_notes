# post_notes 

This is a tool that generates a static website out of my notes. The idea originates from the concept of **digital gardens**. To summarize: A digital garden is a personal space online where you can share and develop your ideas, it tends to be way less structured as a traditional blog.  

> It's called a garden because it is a lot more iterative then a traditional blog. Like a real garden you need to maintain regularly, a digital garden also encourages you to revisit and refine ideas over a prolonged period of time. 

## Keybindings

Since these can deviate quite a lot from the keybindings you might expect, depending on your personal background, I try to keep a up to date list of the most important keybindings below:

### Search

**"/"**: Search through files.
	- **"esc"**: Exit out of search.

### Navigation

**"j"**: Scroll down.
**"k"**: Scroll up. 

## Current Development

- [ ] **CI-Pipeline**: Build and push the static website to from my notes repo to a post_notes_output repo.
- [ ] **Borrow smarter**: There are a lot of strings that get cloned around for convenience. This isn't a huge issue since the project scope is fairly small, but thinking about the right types to use, or simply wrapping some of the parameters that get passed around inside an Arc would do the trick here.
- [ ] **Search**: I've already implemented the search functionality, but it's shitty, so I have to revisit that one.
- [ ] **Modules**: I've gone for the approach where modules register them self in a module registry, which ensures that we do not accidentally register the same keybindings or modules twice. I think this was a interesting idea worth exploring, but going with a explicit structure where each module is called in some sort of `main.js` would make more sense here.
 

TODO! Since this is currently also the only README I have it's content is less concise than I want it to be. This should focus more on stuff related to the development of this tool and less on the project as a whole. That's why I'm going to split this into the README for 

## Technical details

### Note Structure

The most important detail for now is the front matter. This is being used to determine which files to parse/ build.

**Example**:

```md
---
title: Homepage
description: Description
image: 
tags:
  - area/hobby
  - project/post-notes
public: true
modified: 2025-06-25T23:23
created: 2025-05-23T13:35
---
```

## License

This project is licensed under the GNU General Public License v3.0.
