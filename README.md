<div align="center">
  <h1>Post Notes 🦕</h1>
  <div>
    <img alt="GitHub License" src="https://img.shields.io/github/license/Tim-Raphael/post-notes">
    <img alt="GitHub Issues" src="https://img.shields.io/github/issues/Tim-Raphael/post-notes">
    <img alt="GitHub Commit Activity" src="https://img.shields.io/github/commit-activity/m/Tim-Raphael/post-notes">
    <img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/Tim-Raphael/post-notes/release.yml">
  </div>
  <p><strong>Building a cute digital garden.</strong></p>
</div>

  This is a tool that generates a static website out of my notes. The idea originates from the concept of **digital gardens**. To summarize: A digital garden is a personal space online where you can share and develop your ideas, it tends to be way less structured as a traditional blog.

  > It's called a garden because it is a lot more iterative then a traditional blog. Like a real garden you need to maintain regularly, a digital garden also encourages you to revisit and refine ideas over a prolonged period of time.

## Preview

### Website

<img alt="Post Notes Website" src="https://github.com/user-attachments/assets/678cbc00-4e92-4895-af3c-f11f0ab1fae2" />

If you want to look around: [Preview Website](https://tim-raphael.github.io/post-notes-output/)

### Misc 

<details>
  <summary>View Image: Build Process</summary>
  <img alt="Build Process" src="https://github.com/user-attachments/assets/f4da2c52-33d1-4842-842a-1dab59a276a9" />
</details>

## Technical detail

### Note Structure

The most important detail for now is the front matter. This is being used to determine which files to parse/build the website.

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

This project is licensed under the [GNU General Public License v3.0](LICENSE).
