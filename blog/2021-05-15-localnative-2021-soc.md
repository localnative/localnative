---
title: "Local Native 2021 SoC 5/15"
author: Conan Lee
authorURL: 
authorImageURL: 

categories: ["2021 Summer of Code"]
tags: ["all", "localnative", "2021", "SoC"]
---
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
