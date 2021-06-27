import React from 'react';
import classnames from 'classnames';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import styles from './styles.module.css';

const features = [
  {
    title: <>Fast</>,
    imageUrl: 'img/localnative-web-ext-popup.png',
    description: (
      <>
        Fast free text search. <br/>
        Fast save. <br/>
        Fast sync.
      </>
    ),
  },
  {
    title: <>Decentralized</>,
    imageUrl: 'img/localnative-mobile-android-qr.png',
    description: (
      <>
      Local Native empowers you to save and sync your web bookmarks in local SQLite database without going through any centralized service.
    </>
    ),
  },
  {
    title: <>Cross Platform</>,
    imageUrl: 'img/localnative-desktop-0.3.10-qrcode.jpg',
    description: (
      <>
      Sync between multiple devices you own (desktop or mobile) by scanning QR code generated on the same network (Wi-Fi/LAN).
      Search, create, read, delete and pagination.
      Time-series filtering on time range.
      Tag cloud visualization.      </>
    ),
  },
];

function Feature({imageUrl, title, description}) {
  const imgUrl = useBaseUrl(imageUrl);
  return (
    <div className={classnames('col col--4', styles.feature)}>
      {imgUrl && (
        <div className="text--center">
          <img className={styles.featureImage} src={imgUrl} alt={title} />
        </div>
      )}
      <h3>{title}</h3>
      <p>{description}</p>
    </div>
  );
}

function Home() {
  const context = useDocusaurusContext();
  const {siteConfig = {}} = context;
  return (
    <Layout
      title={`${siteConfig.title}`}
      description="Fast, Decentralized and Cross Platform bookmark tool <head />">
      <header className={classnames('hero hero--primary', styles.heroBanner)}>
        <div className="container">
          <h1 className="hero__title">{siteConfig.title}</h1>
          <p className="hero__subtitle">{siteConfig.tagline}</p>

          <div className={styles.buttons}>
          <Link
    className={classnames(
      'button button--outline button--secondary button--lg',
      styles.getStarted,
    )}
    to="https://gitlab.com/localnative/localnative-release/tree/master/v0.5.0">
    Windows
          </Link>
          </div>

          <div className={styles.buttons}>
          <Link
    className={classnames(
      'button button--outline button--secondary button--lg',
      styles.getStarted,
    )}
    to="https://itunes.apple.com/us/app/local-native/id1443968309">
    iOS & iPadOS
          </Link>
          <Link
            className={classnames(
              'button button--outline button--secondary button--lg',
              styles.getStarted,
            )}
            to="https://gitlab.com/localnative/localnative-release/tree/master/v0.5.0/mac">
            macOS
          </Link>
          </div>

          <div className={styles.buttons}>
          <Link
            className={classnames(
              'button button--outline button--secondary button--lg',
              styles.getStarted,
            )}
            to="https://play.google.com/store/apps/details?id=app.localnative">
            Android
          </Link>
          <Link
            className={classnames(
              'button button--outline button--secondary button--lg',
              styles.getStarted,
            )}
            to="https://gitlab.com/localnative/localnative-release/tree/master/v0.5.0/gnu-linux">
              GNU/Linux
          </Link>
          </div>

          <div className={styles.buttons}>
          <Link
            className={classnames(
              'button button--outline button--secondary button--lg',
              styles.getStarted,
            )}
            to="https://addons.mozilla.org/en-US/firefox/addon/localnative">
              Firefox Addon
          </Link>
          <Link
            className={classnames(
              'button button--outline button--secondary button--lg',
              styles.getStarted,
            )}
            to="https://chrome.google.com/webstore/detail/local-native/oclkmkeameccmgnajgogjlhdjeaconnb">
              Chrome Extension
          </Link>
          </div>

          <div class="container">
          <iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/DBsVscpSp6w" frameBorder="0" allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture" allowFullScreen class="video"></iframe>
          </div>
        </div>

          <div class="container">
            <div><h3> WeChat </h3> </div>
            <img src="img/wechat/localnative-wechat-qrcode_344.jpg" width="150" />
          </div>
      </header>
      <main>
        {features && features.length && (
          <section className={styles.features}>
            <div className="container">
              <div className="row">
                {features.map((props, idx) => (
                  <Feature key={idx} {...props} />
                ))}
              </div>
            </div>
          </section>
        )}
      </main>
    </Layout>
  );
}

export default Home;
