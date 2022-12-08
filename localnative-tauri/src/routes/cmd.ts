import { invoke } from '@tauri-apps/api/tauri';
import { emit } from '@tauri-apps/api/event';
import { debounce } from 'underscore';

export function cmdInsertImage(dataURL: any) {
	const message = {
		action: 'insert-image',

		title: 'Screenshot_' + new Date().toISOString(),
		url: 'mime://image/png',
		tags: 'screenshot',
		description: '',
		comments: '',
		annotations: dataURL,

		limit: globalThis.AppState.limit,
		offset: globalThis.AppState.offset,
		is_public: false
	};

	cmd(message);
}

export function cmdSearchOrFilter() {
	const range = globalThis.AppState.range;
	if (range) {
		cmdFilter('', range[0], range[1]);
	} else {
		const message = {
			action: 'select',
			limit: globalThis.AppState.limit,
			offset: globalThis.AppState.offset
		};

		cmd(message);
	}
}

export const cmdFilter = debounce((searchText: string, from: any, to: any) => {
	const message = {
		action: 'filter',
		query: searchText,
		limit: globalThis.AppState.limit,
		offset: globalThis.AppState.offset,
		from: from,
		to: to
	};
	cmd(message);
}, 500);

export function cmdDelete(searchText: string, rowid: number) {
	const message = {
		action: 'delete',
		query: searchText,
		rowid: rowid,
		limit: globalThis.AppState.limit,
		offset: globalThis.AppState.offset
	};

	cmd(message);
}

export function makeTags(str: string) {
	const s = str.replace(/,+/g, ' ').trim();
	const l = s.replace(/\s+/g, ',').split(',');
	const collection = new Set<string>();

	l.forEach(function (tag) {
		collection.add(tag);
	});

	const arr = new Array<string>();
	collection.forEach((v) => arr.push(v));

	return arr.join(',');
}

export function cmdInsert(
	title: string,
	url: string,
	tags_text: string,
	tags_desc: string,
	annotations: any,
	is_public: any
) {
	const message = {
		action: 'insert',
		title: title,
		url: url,
		tags: makeTags(tags_text),
		description: tags_desc,
		comments: '',
		annotations: annotations,

		limit: globalThis.AppState.limit,
		offset: globalThis.AppState.offset,
		is_public: is_public
	};
	console.log(message);
	cmd(message);
}

export function cmdSearchImp(searchText: string) {
	globalThis.AppState.clearOffset();
	globalThis.AppState.range = null;

	const message = {
		action: 'search',

		query: searchText,
		limit: globalThis.AppState.limit,
		offset: globalThis.AppState.offset
	};
	cmd(message);
}

export function cmdSelect() {
	globalThis.AppState.clearOffset();
	globalThis.AppState.range = null;

	const message = {
		action: 'select',
		limit: globalThis.AppState.limit,
		offset: globalThis.AppState.offset
	};

	cmd(message);
}

export function cmdSsbSync() {
	const message = {
		action: 'ssb-sync'
	};
	cmd(message);
}

export function cmdSyncViaAttach(uri: string) {
	const message = {
		action: 'sync-via-attach',
		uri: uri
	};
	cmd(message);
}

export function cmdServer() {
	const message = {
		action: 'server',
		addr: '0.0.0.0:2345'
	};
	cmd(message);
}

export function cmdClientSync(addr: string) {
	const message = {
		action: 'client-sync',
		addr: addr
	};
	cmd(message);
}

export function cmdClientStopServer(addr: string) {
	const message = {
		action: 'client-stop-server',
		addr: addr
	};
	cmd(message);
}

function cmd(message: any) {
	const input = JSON.stringify(message, null, 2);

	invoke<string>('input', { input }).then((res) => {
		const resp: { days: any; notes: any; tags: any; count: number } = JSON.parse(res);

		if (resp.count) {
			globalThis.AppState.count = resp.count;
		}

		onNativeMessage(resp);
	});
}

function onNativeMessage(message: { days: any; notes: any; tags: any; count: number }) {
	emit('refreshTags', { tags: message.tags });

	if (!message.notes) return;

	// update pagination text
	if (message.count >= 0) {
		emit('refreshPaginationText', { pagination_text: globalThis.AppState.makePaginationText() });
	}

	emit('refreshNotes', { notes: message.notes });

	if (
		message.days && // filter result has no days field
		globalThis.AppState.offset == 0 // only first page refresh chart
	) {
		emit('refreshChart', { days: message.days });
	}
}
