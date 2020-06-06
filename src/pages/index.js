import React from 'react';
import classnames from 'classnames';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import styles from './styles.module.css';

const features = [
  {
    title: <>Easy to Use</>,
    imageUrl: 'img/undraw_docusaurus_mountain.svg',
    description: (
      <>
        Docusaurus was designed from the ground up to be easily installed and
        used to get your website up and running quickly.
      </>
    ),
  },
  {
    title: <>Focus on What Matters</>,
    imageUrl: 'img/undraw_docusaurus_tree.svg',
    description: (
      <>
        Docusaurus lets you focus on your docs, and we&apos;ll do the chores. Go
        ahead and move your docs into the <code>docs</code> directory.
      </>
    ),
  },
  {
    title: <>Powered by React</>,
    imageUrl: 'img/undraw_docusaurus_react.svg',
    description: (
      <>
        Extend or customize your website layout by reusing React. Docusaurus can
        be extended while reusing the same header and footer.
      </>
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
      title={`Hello from ${siteConfig.title}`}
      description="Description will go into a meta tag in <head />">
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
    to="https://itunes.apple.com/us/app/local-native/id1443968309">
    iOS & iPadOS
          </Link>
          <Link
            className={classnames(
              'button button--outline button--secondary button--lg',
              styles.getStarted,
            )}
            to="https://gitlab.com/localnative/localnative-release/tree/master/v0.4.1/mac">
            macOS
          </Link>
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
            to="https://gitlab.com/localnative/localnative-release/tree/master/v0.4.1/gnu-linux">
              GNU/Linux
          </Link>
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
