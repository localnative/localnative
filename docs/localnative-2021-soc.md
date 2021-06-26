---
id: localnative-2021-soc
title: Local Native 2021 SoC
---

### 2021-06-26
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/_sIxnoAAep0" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish
  - 2nd view for timeseries visualization
  
- yi started tutorial translation
- Hill to review and improve current tutorial content

### 2021-06-19
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/jdVv5ds0OoQ" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish 
  - progress and code review on timeseries visualization using Canvas

### 2021-06-12
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/Zv5SzXtRA5k" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish 
  - made more progress on tutorial
  - decide to use `iced_aw` library
 
- yi 
  - released v0.5.0 for mobile and Electron
  - show educative UI
    - markdown, terminal and Live App
    - docker environment for interactive content

### 2021-06-05
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/556GzzsbzuE" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish made progress on tutorial
  - start new code with tutorial content
  - plan to replace the old code directory with new code

TODO
- Canon to translate tutorial content on educative

### 2021-05-29
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/MGcQy0IYvm8" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish
  - progress on alert popup, may need another dependency
  - draft of tutorial beginning sections
  - review CI/CD

- yi
  - maintain the balance of short term and long term documentation

TODO
- Cupnfish
  - may use simplified implementation for time range ui
  - write Chinese version of tutorial under `docs`

### 2021-05-22
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/v6O09ZX9l0E" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish
  - demo sync works between wsl and windows
  - cargo xtask release
    - wrapping shell scripts for different platforms
  - alert popup may or may not need another dependency

- yi
  - prefer to use the same version number for all release targets

TODO
- Cupnfish
  - start on time range ui
  - tutorial structure

### 2021-05-15
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/BV54CD_0G1I" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish
  - Resolve "crash on sync via attach file on Mac 11.3.1 big sur"
  - fix: remove icons frome package of release
  - feat: opengl features support
  - feat: drag files to a window to sync via files
  - fix: fix add icons and locales to package

  - memory performance tuning: svg to font
  - demo sync with windows sandbox

  - time range filter feature
    - may implement a ui control for timeseries chart in iced

  - i18n progress

  - sqlite3 conn wrap to implement Sync
    - may related to tags sorting on another thread works


TODO
- Cupnfish
  - create translation placeholder items in `localnative-app-docusaurus-2` in Gitlab Issues for Canan
  - popup alert with cancel/ok button when clicked on delete
  - bump version to 0.5.0 and add localnative_iced in set-version script

- Hill
  - release script for iced app on windows
  - release script for iced app on mac

- Conan
  - Change by separating multiple jobs
  - Make merge_request trigger pipeline work

### 2021-05-08
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/r5xlfirTJf0" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish
  - Add log section to GUI displaying op information
  - fix open server and close server
  - iced specific sync functions
  - i18n WIP

- yi
  - suggest moving pagination UI component up

- Conan
  - Start ci/cd pipeline for `localnative-rs`

TODO
- Cupnfish
  - record video for sync on Windows using Sandbox

- Hill
  - prepare PR for release script to package iced and web ext targets together

- Conan
  - Change by separating multiple jobs
  - Make merge_request trigger pipeline work

### 2021-05-01
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/O4QkEvdRR0o" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish
  - decide to simplify release as compressed file
    - keep bundle code for tutorial
  - sync code change
    - tarpc upgrade
    - drop conn
  - log level adjustment
  - intend to do i18n and date filter

- yi
  - merged the large PR
  - show `localnative_core` crate-type for android and ios caveat
  - crash when sync on ios and android

TODO
- Cupnfish
  - 计划 Blog

- Hill
  - release script to package iced and web ext targets together

### 2021-04-24
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/ShZldgz2WTk" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish demo
  - Refactoring on config file
  - Working with localnative_bundle and try to create Registry in windows for browser extension

- Yi Wang
  - wix templates
  - os config branches in build functions
  - windows sandbox

- Conan
  - Research on rust project and try to make it run

- Hill Chen
  - Test on windows and run with localnative_iced
  - Finding missing ddl dependency issues on windows

TODO
- Cupnfish
  - Finish localnative_bundle build for windows
  - Make mac, linux bundle work
  - Add windows ddl dependency in documentation
- Conan
  - Try to work on CI/CD
- Hill Chen
  - Research and work on rust project

### 2021-04-17
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/i0hrXWHO94E" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish demo
  - building windows installation file localnative.msi
  - review on pr changes

- Yi Wang
  - reviewing with Cupnfish every file changes and make suggestion
  - approve changes

TODO
- Cupnfish
  - finish ui and rest of work for localnative_core
  - Merge pr after conan and yi wang review
- Conan
  - Try to build rust project
  - Try to work on CI/CD

### 2021-04-10
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/MaF1WcY7gVk" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish demo
  - breaking down components to smaller modules
  - updates in `localnative_core`
  - stable rust should work
  - note on compiler flag for build
- discussion about user installation experience
  - should be straight forward with minimum friction
- hill suggested tool for network

TODO
- Cupnfish
  - create 2021-soc branch
  - commit `localnative_core` updates
  - current progress as another commit
- yi to explore Gitee

### 2021-04-07
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/clLJglyEV1g" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- yi demo
  - GitLab Issues
    - iteration
    - how-to tag
  - educative new section for Rust GUI
  - localnative.app website code repo
    - `npm run start`

TODO
- Cupnfish break down larger code merge into smaller logical merges
- hill to raise issues with how-to tag

### 2021-04-03
<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/h1Q4qN9hBN4" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

- Cupnfish demo current state of PoC
  - UI
    - wrapping UI component for tags
  - compare druid and iced
    - druid
      - menu integration
      - seems more complex
        - lens concept
      - with i18n support?
    - iced
      - seems easier concepts for learning
      - performance seems good with existing app
      - seems i18n need some more work
- yi demo current installation process using Electron
  - packing and installation of web ext host binary
  - modify manifest json file for browser

TODO
- Cupnfish explore installation options
- Conan explore GitLab issues
