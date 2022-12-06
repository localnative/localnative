import type { BaseTranslation } from '../i18n-types';

const en: BaseTranslation = {
	Nav: {
		Notes: 'Notes',
		Sync: 'Sync',
		About: 'About',
		Settings: 'Settings'
	},
	Notes: {
		SearchPlaceholder: 'Searching Notes...',
		Tags: 'Tags'
	},
	Sync: {
		SyncWithFile: 'Sync with local file',
		SyncWithFileSelect: 'Select File',
		SyncAsClient: 'Sync as Client',
		SyncAsClientPlaceholder: 'Server Address, eg: 127.0.0.1:2345',
		SyncAsServer: 'Sync as Server',
		SyncAsServerLocalAddr: 'Local Server Address: {serverAddress}'
	},
	Settings: {
		Language: 'Language'
	}
};

export default en;
