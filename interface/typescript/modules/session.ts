import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";
import { listen } from "@tauri-apps/api/event";
import { Buffer } from "../utils/buffer";

/** Модуль клиентской сессии */
class SessionModule {
	private currentSessionAddress: HTMLSpanElement | null = null;
	private currentSessionPassword: HTMLSpanElement | null = null;
	private currentPassword: string | null = null;
	private chatContent: HTMLDivElement | null = null;
	private messageCounter: number = 0;

	/** Метод инициализации сессии */
	public async init(): Promise<void> {
		this.currentSessionAddress = document.getElementById("current-session-address") as HTMLSpanElement;
		this.currentSessionPassword = document.getElementById("current-session-password") as HTMLSpanElement;

		this.currentSessionPassword.addEventListener("click", async () => this.currentPassword ? await navigator.clipboard.writeText(this.currentPassword) : null);

		await this.setupDefaultSession();

		document.getElementById("setup-default-session")?.addEventListener("click", async () => await this.setupDefaultSession());

		this.chatContent = document.getElementById("session-chat-content") as HTMLDivElement;

		await listen<number[]>("session:chat", (e) => {
			if (!this.chatContent) return;

			const buf = new Buffer(new Uint8Array(e.payload));
			const sender = buf.readString();
			const message = buf.readString();

			const el = document.createElement("label");
			el.innerText = `${sender}: ${message}`;

			this.chatContent.appendChild(el);
			this.messageCounter++;

			if (this.messageCounter > 100 && this.chatContent.firstChild) this.chatContent.removeChild(this.chatContent.firstChild);
		});

		document.getElementById("send-session-message")?.addEventListener("click", async () => {
			const message = document.getElementById("session-message") as HTMLInputElement;
			if (message.value === "") return;

			const buf = new Buffer();
			buf.writeU8(0x00);
			buf.writeString(message.value);

			const result = await invoke<CommandResult<null>>("send_command", {
				data: buf.toUint8Array()
			});

			if (result.error) {
				logger.log(`Ошибка отправки сообщения в чат сессии: ${result.error}`, "error");
				return;
			}

			message.value = "";
		});
	}

	/** Метод установки сессии по умолчанию */
	private async setupDefaultSession(): Promise<void> {
		logger.log("Установка локальной сессии...", "system");

		if (this.chatContent) this.chatContent.innerHTML = "";
		this.setInfo("?", "");

		const result = await invoke<CommandResult<Array<string>>>("setup_default_session");

		if (!result.data || result.error) {
			logger.log(`Ошибка установки локальной сессии: ${result.error}`, "error");
			return;
		}

		logger.log(`Локальная сессия с ${result.data[0]} успешно установлена`, "system");

		this.setInfo(result.data[0], result.data[1]);

		await this.synchronize();
	}

	/** Вспомогательный метод установки информации */
	private setInfo(address: string, password: string): void {
		this.currentPassword = password === "" ? null : password;
		if (this.currentSessionAddress) this.currentSessionAddress.innerText = address;
		if (this.currentSessionPassword) this.currentSessionPassword.innerText = password === "" ? "No password" : "✱✱✱✱✱✱";
	}

	private async synchronize(): Promise<void> {
		const buf = new Buffer();
		buf.writeU8(0x03);

		const result = await invoke<CommandResult<null>>("send_command", {
			data: buf.toUint8Array()
		});

		if (result.error) logger.log(`Ошибка синхронизации данных: ${result.error}`, "error");
	}
}

const sessionModule = new SessionModule();

export { sessionModule };
