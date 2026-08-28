import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { logger } from "../utils/logger";
import { Buffer } from "../utils/buffer";
import { indexSystem } from "../systems/index";

/** Структура информации о боте */
interface BotProfile {
	status: BotStatus;
	password: string | null;
	email: string | null;
	proxy: {
		ipAddress: string | null;
		proxy: string | null;
		username: string | null;
		password: string | null;
	};
	ping: number;
	health: number;
	registered: boolean;
	logined: boolean;
	captchaCaught: boolean;
	group: string;
}

enum BotStatus {
	Waiting,
	Connected,
	Disconnected,
}

/** Модуль управления мониторингом */
class MonitoringModule {
	private indexTable: number[] = [];

	private statusText: HTMLElement | null = null;
	private cards: HTMLElement | null = null;
	private wrappers: HTMLElement | null = null;

	private chatMessageCounter: Record<string, number> = {};
	private chatHistoryFilters: Record<string, string> = {};

	public maxChatHistoryLength: number = 0;

	/** Метод инициализации функций, связанных с мониторингом */
	public async init(): Promise<void> {
		this.statusText = document.getElementById("monitoring-status-text");
		this.cards = document.getElementById("bot-cards-container");
		this.wrappers = document.getElementById("bot-wrappers-container");

		this.statusText!.style.display = "flex";

		await listen<number[]>("monitoring:chat", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				const index = buf.readU8();
				const message = buf.readString();

				if (!this.chatHistoryFilters[index]) this.chatHistoryFilters[index] = "all";
				if (!this.filterMessage(this.chatHistoryFilters[index], message)) return;

				const chat = document.getElementById(`monitoring-chat-content-${index}`);
				if (!chat) return;

				const line = document.createElement("div");
				line.className = "line";
				line.setAttribute("cmi", index.toString());
				line.innerHTML = message;

				chat.appendChild(line);

				if (!this.chatMessageCounter[index]) {
					this.chatMessageCounter[index] = 1;
				} else {
					this.chatMessageCounter[index]++;
					if (this.chatMessageCounter[index] > this.maxChatHistoryLength) {
						this.chatMessageCounter[index]--;
						chat.firstChild?.remove();
					}
				}
			} catch (error) {
				logger.log(`Ошибка обработки события "monitoring:chat": ${error}`, "error");
			}
		});

		await listen<number[]>("monitoring:update-profile", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				const index = buf.readU8();

				const username = indexSystem.usernameByIndex(index);
				if (!username) return;

				const statusCode = buf.readU8();
				let status = BotStatus.Waiting;

				switch (statusCode) {
					case 0x01:
						status = BotStatus.Connected;
						break;
					case 0x02:
						status = BotStatus.Disconnected;
						break;
				}

				const profile: BotProfile = {
					status: status,
					password: buf.readOption(() => buf.readString()),
					email: buf.readOption(() => buf.readString()),
					proxy: {
						ipAddress: buf.readOption(() => buf.readString()),
						proxy: buf.readOption(() => buf.readString()),
						username: buf.readOption(() => buf.readString()),
						password: buf.readOption(() => buf.readString()),
					},
					ping: buf.readU32(),
					health: buf.readU32(),
					registered: buf.readBoolean(),
					logined: buf.readBoolean(),
					captchaCaught: buf.readBoolean(),
					group: buf.readString(),
				};

				this.indexTable.includes(index) ? this.updateBotCard(index, profile) : this.createBotCard(index, username, profile);
			} catch (error) {
				logger.log(`Ошибка обработки события "monitoring:update-profile": ${error}`, "error");
			}
		});
	}

	/** Метод включения модуля */
	public enable(): void {
		this.statusText!.style.display = "none";
		this.cards!.innerHTML = "";
		this.cards!.style.display = "flex";
	}

	/** Метод очистки и выключения мониторинга */
	public disable(): void {
		document.querySelectorAll('[wrapper="bot-chat"]').forEach(w => w.remove());
		document.querySelectorAll('[wrapper="bot-card"]').forEach(w => w.remove());

		this.cards!.innerHTML = "";
		this.cards!.style.display = "none";
		this.wrappers!.innerHTML = "";

		this.chatMessageCounter = {};
		this.chatHistoryFilters = {};
		this.indexTable = [];

		this.statusText!.style.display = "flex";
	}

	/** Метод создания карточки бота */
	private createBotCard(index: number, username: string, profile: BotProfile): void {
		const steveIconPath = document.getElementById("steve-img") as HTMLImageElement;

		const card = document.createElement("div");
		card.className = "profile";
		card.id = `profile-${index}`;
		card.setAttribute("wrapper", "bot-card");

		card.innerHTML = `
      <div class="head">
        <div class="top">
          <img src="${steveIconPath.src}" class="image" draggable="false">
          <div class="text">
            <div class="username">${username}</div>
            <div class="status" id="monitoring-status-${index}">Waiting</div>
          </div>
        </div>

        <div class="bottom">
          <input type="text" class="bot-group" id="bot-group-${index}" placeholder="bot group">
        </div>
      </div>

      <div class="info">
        <p class="line">Пинг: <span id="monitoring-ping-${index}">${profile.ping}</span>ms</p>
        <p class="line">Здоровье: <span id="monitoring-health-${index}">${profile.health}</span> / 20</p>
        <p class="line">Прокси: <span id="monitoring-proxy-${index}">${profile.proxy.ipAddress ?? "?"}</span></p>
        <p class="line">Пароль: <span>${profile.password ?? "No password"}</span></p>
      </div>

      <div class="buttons">
        <button class="min" id="open-chat-${index}">Открыть чат</button>
        <button class="min" id="reset-${index}">Сбросить</button>
        <button class="min" id="disconnect-${index}">Отключить</button>
      </div>
    `;

		this.cards?.appendChild(card);
		this.initializeBotCard(index, username);
	}

	/** Метод создания обёркти чата у бота */
	private createChatWrapper(index: number, username: string): HTMLDivElement {
		const wrapper = document.createElement("div");
		wrapper.className = "cover";
		wrapper.id = `chat-${index}`;
		wrapper.setAttribute("wrapper", "bot-chat");

		wrapper.innerHTML = `
      <div class="panel">
        <div class="left">
          <div class="custom-select">
            <select id="select-chat-filter-${index}">
              <option value="all">Все сообщения</option>
              <option value="bans">Блокировки</option>
              <option value="mentions">Упоминания</option>
              <option value="warnings">Предупреждения</option>
              <option value="links">Ссылки</option>
            </select>
          </div>

          <button class="min" id="filter-chat-${index}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-filter">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M4 4h16v2.172a2 2 0 0 1 -.586 1.414l-4.414 4.414v7l-6 2v-8.5l-4.48 -4.928a2 2 0 0 1 -.52 -1.345v-2.227" />
            </svg>
          </button>
        </div>

        <div class="right">
          <button class="min" id="clear-chat-${index}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-trash">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M4 7l16 0" />
              <path d="M10 11l0 6" />
              <path d="M14 11l0 6" />
              <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" />
              <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />
            </svg>
          </button>

          <button class="min" id="close-chat-${index}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M18 6l-12 12" />
              <path d="M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <div class="chat-content" id="monitoring-chat-content-${index}"></div>

      <div class="pretty-input-wrapper">
        <p class="signature">${username}</p>
        <input type="text" id="chat-message-${index}" placeholder="Введите и отправьте сообщение, нажав «Enter»">
      </div>
    `;

		this.wrappers?.appendChild(wrapper);

		return wrapper;
	}

	/** Метод обновления карточки бота */
	private updateBotCard(index: number, profile: BotProfile): void {
		const status = document.getElementById(`monitoring-status-${index}`) as HTMLSpanElement | undefined;
		if (!status) return;

		const proxy = document.getElementById(`monitoring-proxy-${index}`) as HTMLSpanElement | undefined;
		if (!proxy) return;

		const ping = document.getElementById(`monitoring-ping-${index}`) as HTMLSpanElement | undefined;
		if (!ping) return;

		const health = document.getElementById(`monitoring-health-${index}`) as HTMLSpanElement | undefined;
		if (!health) return;

		const group = document.getElementById(`bot-group-${index}`) as HTMLInputElement | undefined;
		if (!group) return;

		if (health.innerText.split("/")[0].replace(" ", "") != profile.health.toString()) {
			const card = document.getElementById(`profile-${index}`);
			card?.classList.add("glow");
			setTimeout(() => card?.classList.remove("glow"), 300);
		}

		let statusColor = "var(--bot-waiting-status-color)";

		switch (profile.status) {
			case BotStatus.Waiting:
				status.innerText = "Waiting";
				statusColor = "var(--bot-waiting-status-color)";
				break;
			case BotStatus.Connected:
				status.innerText = "Connected";
				statusColor = "var(--bot-connected-status-color)";
				break;
			case BotStatus.Disconnected:
				status.innerText = "Disconnected";
				statusColor = "var(--bot-disconnected-status-color)";
				break;
		}

		status.style.color = statusColor;
		proxy.innerText = profile.proxy.ipAddress ?? "No proxy";
		ping.innerText = profile.ping.toString();
		health.innerText = profile.health.toString();

		// Если группа по умолчанию её можно и не отображать, это ни на что не повлияет
		group.value = profile.group == "global" ? "" : profile.group;
	}

	/** Метод инициализации карточки бота */
	private initializeBotCard(index: number, username: string): void {
		this.chatHistoryFilters[index] = "all";
		this.indexTable.push(index);

		const chatWrapper = this.createChatWrapper(index, username);

		document.getElementById(`open-chat-${index}`)?.addEventListener("click", () => chatWrapper.style.display = "flex");
		document.getElementById(`close-chat-${index}`)?.addEventListener("click", () => chatWrapper.style.display = "none");

		document.getElementById(`chat-${index}`)?.addEventListener("keydown", async (e) => {
			if ((e as KeyboardEvent).key === "Enter") {
				const message = document.getElementById(`chat-message-${index}`) as HTMLInputElement;

				const buf = new Buffer();
				buf.writeU8(0x05);
				buf.writeU8(0x00);
				buf.writeU8(index);
				buf.writeString(message.value);

				const result = await invoke<CommandResult<null>>("send_command", {
					data: buf.toUint8Array(),
				});

				if (result.error) {
					logger.log(`Ошибка отправки сообщения от бота ${username}: ${result.error}`, "error");
					return;
				}

				message.value = "";
			}
		});

		document.getElementById(`reset-${index}`)?.addEventListener("click", async () => {
			try {
				const buf = new Buffer();
				buf.writeU8(0x05);
				buf.writeU8(0x01);
				buf.writeU8(index);

				const result = await invoke<CommandResult<null>>("send_command", {
					data: buf.toUint8Array(),
				});

				if (result.error) logger.log(`Ошибка сбрасывания задач и состояний бота ${username}: ${result.error}`, "error");
			} catch (error) {
				logger.log(`Ошибка сбрасывания задач и состояний бота ${username}: ${error}`, "error");
			}
		});

		document.getElementById(`disconnect-${index}`)?.addEventListener("click", async () => {
			try {
				const buf = new Buffer();
				buf.writeU8(0x05);
				buf.writeU8(0x02);
				buf.writeU8(index);

				const result = await invoke<CommandResult<null>>("send_command", {
					data: buf.toUint8Array(),
				});

				if (result.error) logger.log(`Ошибка отключения бота ${username}: ${result.error}`, "error");
			} catch (error) {
				logger.log(`Ошибка отключения бота ${username}: ${error}`, "error");
			}
		});

		document.getElementById(`bot-group-${index}`)?.addEventListener("input", async () => {
			try {
				const group = (document.getElementById(`bot-group-${index}`) as HTMLInputElement).value.replace(" ", "") || "global";

				const buf = new Buffer();
				buf.writeU8(0x06);
				buf.writeU8(index);
				buf.writeString(group);

				const result = await invoke<CommandResult<null>>("send_command", {
					data: buf.toUint8Array(),
				});

				if (result.error) logger.log(`Ошибка изменения группы бота ${username}: ${result.error}`, "error");
			} catch (error) {
				logger.log(`Ошибка изменения группы бота ${username}: ${error}`, "error");
			}
		});

		document.getElementById(`filter-chat-${index}`)?.addEventListener("click", () => {
			try {
				const content = document.getElementById(`monitoring-chat-content-${index}`);
				const type = document.getElementById(`select-chat-filter-${index}`) as HTMLSelectElement;
				const history = [...document.querySelectorAll(`[cmi="${index}"]`).values()];

				content!.innerHTML = "";
				this.chatHistoryFilters[index] = type.value;

				history.forEach(m => this.filterMessage(type.value, m.textContent || "") ? content?.appendChild(m) : null);
			} catch (error) {
				logger.log(`Ошибка фильтровки чата у бота ${username}: ${error}`, "error");
			}
		});

		document.getElementById(`clear-chat-${index}`)?.addEventListener("click", () => {
			const messages = document.querySelectorAll(`[cmi="${index}"]`);
			messages.forEach(msg => msg.remove());
			this.chatMessageCounter[index] = 0;
		});
	}

	/** Метод создания триграмм из слова */
	private createTrigrams(word: string): string[] {
		const trigrams = [];
		for (let i = 0; i <= word.length - 3; i++) trigrams.push(word.substring(i, 3));
		return trigrams;
	}

	/** Метод проверки слова на наличие определённых паттернов */
	private checkPatterns(word: string, patterns: string[]): boolean {
		if (word.length < 3) return false;

		let totalTrigrams = 0;
		let similarTrigrams = 0;

		const wts = this.createTrigrams(word);
		totalTrigrams = wts.length;

		for (const p of patterns) for (const wt of wts) for (const pt of this.createTrigrams(p)) wt.toLowerCase() == pt.toLowerCase() ? similarTrigrams++ : null;

		if (similarTrigrams >= totalTrigrams / 2) return true;

		return false;
	}

	/** Метод фильтровки сообщения */
	private filterMessage(type: string, message: string): boolean {
		if (type === "all") {
			return true;
		} else if (type === "links") {
			const patterns = [
				"http://", "https://", ".dev", ".com", ".org",
				".io", ".ai", ".net", ".pro", ".gov", ".lv",
				".ru", ".onion", ".ie", ".co", ".fun", ".gg",
				".xyz", ".club", ".eu", ".me", ".us", ".online",
				".br", ".cc", ".no"
			];

			let result = false;

			for (const p of patterns) {
				if (message.includes(p)) {
					result = true;
					break;
				}
			}

			return result;
		} else {
			const patterns: Record<string, string[]> = {
				bans: [
					"banned", "ban", "kicked",
					"kick", "кикнут", "заблокированный",
					"заблокирован", "блокировка",
					"заблокировали", "забанен",
					"забанили", "бан", "blocked"
				],
				warnings: [
					"предупреждение", "warn", "warning",
					"важно", "important", "предупреждает",
					"важная", "уведомление", "осведомление",
					"notice", "уведомлять", "замечание"
				],
				mentions: [
					"упомянут", "mention", "reference",
					"упоминает", "упоминание", "ссылаться"
				]
			};

			let results: boolean[] = [];
			for (const word of message.split(" ")) results.push(this.checkPatterns(word, patterns[type]));
			if (results.includes(true)) return true;
		}

		return false;
	}
}

const monitoringModule = new MonitoringModule();

export { monitoringModule }
