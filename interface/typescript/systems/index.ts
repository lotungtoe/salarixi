import { Buffer } from "../utils/buffer";

/**
 * Система индексации ботов.
 * 
 * Работает аналогично системе индексации в бэке.
 * У этой системы две задачи: синхронизация индексов
 * и выдача юзернейма по индексу.
 * 
 * Синхронизация происходит через сессионные события.
 * */
class IndexSystem {
	private table: Array<string> = [];

	/** Метод синхронизации индексов с бэком */
	public synchronize(buf: Buffer): void {
		const len = buf.readU8();
		const table = [];

		for (let i = 0; i < len; i++) {
			const username = buf.readString();
			table[i] = username;
		}

		this.table = table;
	}

	/** Метод получения юзернейма по индексу */
	public usernameByIndex(index: number): string | undefined {
		return this.table[index];
	}

	/** Метод очистки таблицы индексов */
	public clear(): void {
		this.table = [];
	}
}

const indexSystem = new IndexSystem();

export { indexSystem };
