// @ts-check
const lightCodeTheme = require('prism-react-renderer').themes.github;
const darkCodeTheme = require('prism-react-renderer').themes.dracula;

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Local Native',
  tagline: 'Own your bookmarks on your device.',
  url: 'https://localnative.app',
  baseUrl: '/',
  favicon: 'img/icon.png',
  organizationName: 'localnative',
  projectName: 'localnative',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://github.com/localnative/localnative/tree/main/website/',
        },
        blog: {
          showReadingTime: true,
          editUrl: 'https://github.com/localnative/localnative/tree/main/website/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],
  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        disableSwitch: true,
        defaultMode: 'dark',
      },
      navbar: {
        title: 'Local Native',
        logo: {
          alt: 'Local Native Logo',
          src: 'img/icon.png',
        },
        items: [
          {
            to: 'docs/quick-start',
            activeBasePath: 'docs',
            label: 'Docs',
            position: 'left',
          },
          {
            to: 'docs/developer-setup',
            activeBasePath: 'docs',
            label: 'Developer',
            position: 'left',
          },
          {to: 'blog', label: 'Blog', position: 'left'},
          {to: 'privacy-policy', label: 'Privacy', position: 'left'},
          {
            to: 'docs/lecture',
            activeBasePath: 'docs',
            label: 'Lecture',
            position: 'right',
          },
          {
            to: 'docs/talk',
            activeBasePath: 'docs',
            label: 'Talk',
            position: 'right',
          },
          {
            href: 'https://github.com/localnative/localnative',
            position: 'right',
            className: 'header-github-link',
            'aria-label': 'GitHub repository',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'WeChat',
            items: [
              {
                html: `<img src="img/wechat/localnative-wechat-qrcode_344.jpg" width="100" />`
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Open Collective',
                href: 'https://opencollective.com/localnative',
              },
              {
                label: 'Librem Social',
                href: 'https://social.librem.one/@yi',
              },
            ],
          },
          {
            items: [
              {
                label: 'YouTube',
                href: 'https://www.youtube.com/@localnative',
              },
              {
                label: 'Twitter',
                href: 'https://twitter.com/localnative_app',
              },
              {
                label: 'Facebook',
                href: 'https://www.facebook.com/localnativeapp',
              },
            ],
          },
          {
            title: 'Docs',
            items: [
              {
                label: 'Quick Start',
                to: 'docs/quick-start',
              },
              {
                label: 'Developer Setup',
                to: 'docs/developer-setup',
              },
              {
                label: 'Privacy Policy',
                to: 'privacy-policy',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'Blog',
                to: 'blog',
              },
              {
                label: 'GitHub',
                href: 'https://github.com/localnative/localnative',
              },
            ],
          },
        ],
        copyright: `Unless otherwise noted, contents on this website are copyleft with a CC-by-SA 4.0 license.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
      },
    }),
};

module.exports = config;
