declare global {
	// eslint-disable-next-line no-var
	var AppState: State;
}

export class State {
	private _limit = 10;
	private _range: [number, number] | null = null;
	private _query = '';
	private _count = 0;
	private _offset = 0;

	public get limit() {
		return this._limit;
	}

	public get offset(): number {
		return this._offset;
	}

	public get count(): number {
		return this._count;
	}

	public set count(value: number) {
		this._count = value;
	}

	public get query(): string {
		return this._query;
	}
	public set query(value: string) {
		this._query = value;
	}

	public get range(): [number, number] | null {
		return this._range;
	}
	public set range(value: [number, number] | null) {
		this._range = value;
	}

	public makePaginationText(): string {
		const start = this._count > 0 ? this._offset + 1 : 0;
		const end = this._offset + this._limit > this._count ? this._count : this._offset + this._limit;
		return `${start}-${end} / ${this._count}`;
	}

	public incOffset() {
		if (this._offset + this._limit < this._count) {
			this._offset += this._limit;
		}
		return this._offset;
	}

	public decOffset() {
		if (this._offset - this._limit >= 0) {
			this._offset -= this._limit;
		}
		return this._offset;
	}

	public clearOffset() {
		this._offset = 0;
	}
}
