import React from 'react';
import Layout from '@theme/Layout';

function Hello() {
  return (
    <Layout title="Hello">
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '50vh',
          fontSize: '20px',
        }}>

        <div className="post">
  <header className="postHeader">
    <h1>Privacy Policy</h1>
  </header>
<ul>
<li>
As the name "Local Native" implies, the application stores your information locally on your own device.
</li>
<li>
Local Native App developer does not run any service to collect or store any personal information about you.
</li>
<li>
If you choose to sync data via LAN/WiFi to other devices, be aware that is you voluntarily share your data with other peers.
</li>
<li>
Platform providers and underlying operating systems (android, ios, web browsers, app store) may collect information about the application (crash reports, usage stats) and make those avaliable to Local Native App developer, who may use those data to improve the application.
</li>
</ul>
</div>

      </div>
    </Layout>
  );
}

export default Hello;
