export class Buffer {
	private buf: Uint8Array;
	private view: DataView;
	public len = 0;

	constructor() {
		this.buf = new Uint8Array();
		this.view = new DataView(this.buf.buffer);
	}

	private cup(n: number): void {
		const needed = this.len + n;
		if (needed <= this.buf.byteLength) return;

		let newCap = Math.max(this.buf.byteLength * 2, needed);
		const newBuf = new Uint8Array(newCap);
		newBuf.set(this.buf.subarray(0, this.len));

		this.buf = newBuf;
		this.view = new DataView(this.buf.buffer);
	}

	public toUint8Array(): Uint8Array {
		return this.buf.subarray(0, this.len);
	}

	public writeU8(value: number): void {
		this.cup(1);
		this.view.setUint8(this.len, value);
		this.len += 1;
	}

	public writeU16(value: number): void {
		this.cup(2);
		this.view.setUint16(this.len, value);
		this.len += 2;
	}

	public writeU64(value: bigint): void {
		this.cup(8);
		this.view.setBigUint64(this.len, value);
		this.len += 8;
	}

	public writeI32(value: number): void {
		this.cup(4);
		this.view.setInt32(this.len, value);
		this.len += 4;
	}

	public writeI64(value: bigint): void {
		this.cup(8);
		this.view.setBigInt64(this.len, value);
		this.len += 8;
	}

	public writeF32(value: number): void {
		this.cup(4);
		this.view.setFloat32(this.len, value);
		this.len += 4;
	}

	public writeF64(value: number): void {
		this.cup(8);
		this.view.setFloat64(this.len, value);
		this.len += 8;
	}

	public writeBytes(value: Uint8Array): void {
		this.cup(value.byteLength);
		this.buf.set(value, this.len);
		this.len += value.byteLength;
	}

	public writeString(value: string): void {
		const encoder = new TextEncoder();
		const bytes = encoder.encode(value);

		this.writeU16(bytes.byteLength);
		this.writeBytes(bytes);
	}
}
