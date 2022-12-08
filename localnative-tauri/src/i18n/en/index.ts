import type { BaseTranslation } from '../i18n-types';

const en: BaseTranslation = {
	Yes: 'Yes',
	No: 'No',
	Ok: 'Ok',
	Nav: {
		Notes: 'Notes',
		Sync: 'Sync',
		About: 'About',
		Settings: 'Settings'
	},
	Notes: {
		SearchPlaceholder: 'Searching Notes...',
		Tags: 'Tags',
		DeleteModalTitle: 'Delete Note',
		DeleteModalContent: 'Do you want to delete this note?'
	},
	Sync: {
		SyncWithFile: 'Sync with local file',
		SyncWithFileSelect: 'Select File',
		SyncAsClient: 'Sync as Client',
		SyncAsClientPlaceholder: 'Server Address, eg: 127.0.0.1:2345',
		SyncAsServer: 'Sync as Server',
		StartSyncServer: 'Start Server',
		StopSyncServer: 'Stop Server',
		SyncAsServerLocalAddr: 'Local Server Address: {serverAddress}',
		SyncAsClientConnectServerNotExistModalTitle: 'Error',
		SyncAsClientConnectServerNotExistModalContent: 'Connect failed'
	},
	Settings: {
		Language: 'Language'
	}
};

export default en;
