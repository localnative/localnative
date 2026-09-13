import React from 'react';
import clsx from 'clsx';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import styles from './styles.module.css';

const platforms = [
  {
    label: 'Windows',
    to: 'https://gitlab.com/localnative/localnative-release/tree/master/v0.5.1/win',
  },
  {
    label: 'macOS',
    to: 'https://gitlab.com/localnative/localnative-release/tree/master/v0.5.1/mac',
  },
  {
    label: 'GNU/Linux',
    to: 'https://gitlab.com/localnative/localnative-release/tree/master/v0.5.0/gnu-linux',
  },
  {
    label: 'iOS & iPadOS',
    to: 'https://itunes.apple.com/us/app/local-native/id1443968309',
  },
  {
    label: 'Android',
    to: 'https://play.google.com/store/apps/details?id=app.localnative',
  },
  {
    label: 'Firefox Add-on',
    to: 'https://addons.mozilla.org/en-US/firefox/addon/localnative',
  },
  {
    label: 'Chrome Extension',
    to: 'https://chrome.google.com/webstore/detail/local-native/oclkmkeameccmgnajgogjlhdjeaconnb',
  },
];

const features = [
  {
    title: 'Fast',
    imageUrl: 'img/localnative-web-ext-popup.png',
    alt: 'Local Native browser extension popup',
    description: (
      <>
        Fast free text search backed by SQLite FTS. Fast save from the browser
        extension. Fast sync between your own devices.
      </>
    ),
  },
  {
    title: 'Decentralized',
    imageUrl: 'img/localnative-mobile-android-qr.png',
    alt: 'Local Native on Android scanning a sync QR code',
    description: (
      <>
        Local Native empowers you to save and sync your web bookmarks in a
        local SQLite database without going through any centralized service.
      </>
    ),
  },
  {
    title: 'Cross Platform',
    imageUrl: 'img/localnative-desktop-0.3.10-qrcode.jpg',
    alt: 'Local Native desktop app showing a sync QR code',
    description: (
      <>
        Sync between multiple devices you own (desktop or mobile) by scanning
        a QR code on the same network (Wi-Fi/LAN). Time-series filtering, tag
        cloud visualization, full CRUD and pagination.
      </>
    ),
  },
];

function Feature({imageUrl, alt, title, description, index}) {
  const imgUrl = useBaseUrl(imageUrl);
  return (
    <div className={clsx('col col--4', styles.feature)}>
      <span className={styles.featureIndex}>{index}</span>
      {imgUrl && (
        <div className={styles.featureImageWrap}>
          <img className={styles.featureImage} src={imgUrl} alt={alt} />
        </div>
      )}
      <h3 className={styles.featureTitle}>{title}</h3>
      <p className={styles.featureDescription}>{description}</p>
    </div>
  );
}

function Terminal() {
  return (
    <div className={styles.terminal}>
      <div className={styles.terminalBar}>
        <span className={styles.terminalDot} />
        <span className={styles.terminalDot} />
        <span className={styles.terminalDot} />
        <span className={styles.terminalTitle}>localnative — localhost</span>
      </div>
      <div className={styles.terminalBody}>
        <p>
          <span className={styles.prompt}>&gt;&gt;</span>{' '}
          {'{"action":"search","query":"sqlite","limit":10,"offset":0}'}
        </p>
        <p>
          <span className={styles.response}>&lt;&lt;</span>{' '}
          {'{"count":13258,"notes":[{"rowid":13262,"title":"SQLite Query Language: WITH clause"},…]}'}
        </p>
        <p>
          <span className={styles.prompt}>&gt;&gt;</span>{' '}
          {'{"action":"server","addr":"0.0.0.0:2345"}'}
        </p>
        <p>
          <span className={styles.response}>&lt;&lt;</span>{' '}
          {'{"server":"started"}'}{' '}
          <span className={styles.comment}>
            // scan the QR code from another device to sync
          </span>
        </p>
        <p>
          <span className={styles.prompt}>&gt;&gt;</span>{' '}
          <span className={styles.cursor}>&nbsp;</span>
        </p>
      </div>
    </div>
  );
}

function Home() {
  const context = useDocusaurusContext();
  const {siteConfig = {}} = context;
  return (
    <Layout
      title={`${siteConfig.title}`}
      description="Fast, decentralized and cross platform bookmark tool. Own your bookmarks on your device.">
      <header className={styles.heroBanner}>
        <div className="container">
          <p className={styles.kicker}>
            [ local-first · sqlite · p2p sync · no cloud ]
          </p>
          <h1 className={styles.heroTitle}>{siteConfig.title}</h1>
          <p className={styles.heroSubtitle}>{siteConfig.tagline}</p>

          <div className={styles.buttons}>
            <Link
              className={clsx('button button--primary button--lg', styles.cta)}
              to="docs/quick-start">
              Get Started
            </Link>
            <Link
              className={clsx(
                'button button--outline button--secondary button--lg',
                styles.cta,
              )}
              to="https://github.com/localnative/localnative">
              View on GitHub
            </Link>
          </div>

          <Terminal />
        </div>
      </header>

      <main>
        <section className={styles.platforms}>
          <div className="container">
            <h2 className={styles.sectionTitle}>
              <span className={styles.sectionTitleMono}>$</span> install
            </h2>
            <div className={styles.platformGrid}>
              {platforms.map((platform, idx) => (
                <Link
                  key={idx}
                  className={styles.platformCell}
                  to={platform.to}>
                  <span className={styles.platformLabel}>{platform.label}</span>
                  <span className={styles.platformArrow}>↗</span>
                </Link>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.features}>
          <div className="container">
            <h2 className={styles.sectionTitle}>
              <span className={styles.sectionTitleMono}>$</span> why local
              native
            </h2>
            <div className="row">
              {features.map((props, idx) => (
                <Feature
                  key={idx}
                  index={String(idx + 1).padStart(2, '0')}
                  {...props}
                />
              ))}
            </div>
          </div>
        </section>

        <section className={styles.video}>
          <div className="container">
            <h2 className={styles.sectionTitle}>
              <span className={styles.sectionTitleMono}>$</span> see it in
              action
            </h2>
            <div className={styles.videoFrame}>
              <iframe
                src="https://www.youtube-nocookie.com/embed/3dhB5gTtXNM"
                title="Local Native demo video"
                frameBorder="0"
                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                allowFullScreen
              />
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}

export default Home;
