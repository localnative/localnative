module.exports = {
  title: 'Local Native',
  tagline: 'Own your bookmarks on your device.',
  url: 'https://localnative.app',
  baseUrl: '/',
  favicon: 'img/icon.png',
  organizationName: 'Local Native', // Usually your GitHub org/user name.
  projectName: 'Local Native', // Usually your repo name.
  themeConfig: {
    navbar: {
      title: 'Local Native',
      logo: {
        alt: 'Local Native Logo',
        src: 'img/icon.png',
      },
      links: [
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
          href: 'https://gitlab.com/localnative/localnative',
          label: 'GitLab',
          position: 'right',
        },
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
      ],
    },
    footer: {
      style: 'dark',
      links: [
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
          title: 'Community',
          items: [
            {
              label: 'Librem Social',
              href: 'https://social.librem.one/@yi',
            },
            {
              label: 'Open Collective',
              href: 'https://opencollective.com/localnative',
            },
          ],
        },
        {
          title: 'Video',
          items: [
            {
              label: 'YouTube',
              href: 'https://www.youtube.com/channel/UCO3qFIyK0eSmqvMknsslWRw',
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
              label: 'GitLab',
              href: 'https://gitlab.com/localnative/localnative',
            },
          ],
        },
      ],
      copyright: `Unless otherwise noted, contents on this website are copyleft with a CC-by-SA 4.0 license.`,
    },
  },
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          editUrl:
            'https://github.com/facebook/docusaurus/edit/master/website/',
        },
        blog: {
          showReadingTime: true,
          // Please change this to your repo.
          editUrl:
            'https://github.com/facebook/docusaurus/edit/master/website/blog/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
};
