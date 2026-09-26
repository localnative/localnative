import type { Translation } from '../i18n-types';

const zh: Translation = {
	Yes: '是',
	No: '否',
	Ok: '确定',
	Nav: {
		Notes: '便签',
		Sync: '同步',
		About: '关于',
		Settings: '设置'
	},
	Notes: {
		SearchPlaceholder: '搜索标签...',
		Tags: '标签',
		DeleteModalTitle: '删除便签',
		DeleteModalContent: '你想要删除这条便签吗？'
	},
	Sync: {
		SyncWithFile: '通过本地文件同步',
		SyncWithFileSelect: '选择文件',
		SyncAsClient: '作为客户端来同步',
		SyncAsClientPlaceholder: '服务器地址，例如：127.0.0.1:2345',
		SyncAsServer: '作为服务器来同步',
		StartSyncServer: '启动服务器',
		StopSyncServer: '关闭服务器',
		SyncAsServerLocalAddr: '本地服务器地址：{serverAddress}',
		SyncAsClientConnectServerNotExistModalTitle: '错误',
		SyncAsClientConnectServerNotExistModalContent: '连接失败',
		PairingCodePlaceholder: '配对码（首次同步时填写）',
		PairNewDevice: '配对新设备',
		PairingCodeShowing: '配对码：'
	},
	Settings: {
		Language: '语言',
		CheckForUpdates: '检查更新',
		CheckingForUpdates: '正在检查…',
		UpdateUpToDate: '已是最新版本',
		UpdateFailed: '检查更新失败'
	}
};

export default zh;
