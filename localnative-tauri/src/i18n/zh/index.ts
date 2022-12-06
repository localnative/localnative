import type { Translation } from '../i18n-types';

const zh: Translation = {
	Nav: {
		Notes: '便签',
		Sync: '同步',
		About: '关于',
		Settings: '设置'
	},
	Notes: {
		SearchPlaceholder: '搜索标签...',
		Tags: '标签'
	},
	Sync: {
		SyncWithFile: '通过本地文件同步',
		SyncWithFileSelect: '选择文件',
		SyncAsClient: '作为客户端来同步',
		SyncAsClientPlaceholder: '服务器地址，例如：127.0.0.1:2345',
		SyncAsServer: '作为服务器来同步',
		SyncAsServerLocalAddr: '本地服务器地址：{serverAddress}'
	},
	Settings: {
		Language: '语言'
	}
};

export default zh;
