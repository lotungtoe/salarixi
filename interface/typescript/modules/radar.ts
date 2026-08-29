import { invoke } from "@tauri-apps/api/core";
import { isAbsolute } from "@tauri-apps/api/path";

import { logger } from "../utils/logger";
import { messages } from "../utils/message";
import { Buffer } from "../utils/buffer";
import { listen } from "@tauri-apps/api/event";

/** Модуль обнаружения игроков */
class RadarModule {
	private active: boolean = false;

	private targetCardsContainer: HTMLElement | null = null;

	private updateFrequency: number = 1500;
	private targets: Map<string, { data: any, interval: any }> = new Map();

	/** Метод инициализации функций, связанных с радаром */
	public async init(): Promise<void> {
		this.targetCardsContainer = document.getElementById("radar-target-cards-container") as HTMLElement;

		const addTargetBtn = document.getElementById("radar-add-target") as HTMLButtonElement;
		const openSettingsBtn = document.getElementById("radar-open-settings") as HTMLButtonElement;
		const closeSettingsBtn = document.getElementById("radar-close-settings") as HTMLButtonElement;
		const removeAllTargetsBtn = document.getElementById("radar-remove-all-targets") as HTMLElement;
		const updateFrequency = document.getElementById("radar_select_update-frequency") as HTMLSelectElement;

		addTargetBtn.addEventListener("click", () => {
			if (!this.active) return;

			const usernameInput = document.getElementById("radar-target-username") as HTMLInputElement;
			const username = usernameInput.value;

			if (this.targets.has(username)) return;

			usernameInput.value = "";

			this.targets.set(username, { data: null, interval: null });

			const card = document.createElement("div");
			card.className = "radar-target";
			card.id = `radar-target-${username}`;

			card.innerHTML = `
        <svg class="card-icon" xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-person-fill" viewBox="0 0 16 16">
          <path d="M3 14s-1 0-1-1 1-4 6-4 6 3 6 4-1 1-1 1zm5-6a3 3 0 1 0 0-6 3 3 0 0 0 0 6"/>
        </svg>

        <div class="sep"></div>

        <div class="info" style="min-width: 180px;">
          <p>Никнейм: <span>${username.length <= 16 ? username : username.substring(0, 16) + "..."}</span></p>
          <p>Статус: <span id="radar-target-status-${username}">Not found</span></p>
          <p>UUID: <span id="radar-target-uuid-${username}">?</span></p>
        </div>

        <div class="sep"></div>

        <div class="info" style="min-width: 80px;">
          <p>X: <span id="radar-target-x-${username}">?</span></p>
          <p>Y: <span id="radar-target-y-${username}">?</span></p>
          <p>Z: <span id="radar-target-z-${username}">?</span></p>
        </div>

        <div class="sep"></div>

        <div class="btn-group-flex" style="margin-top: 0;">
          <button class="min" id="radar-follow-target-${username}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-current-location">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M9 12a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
              <path d="M4 12a8 8 0 1 0 16 0a8 8 0 1 0 -16 0" />
              <path d="M12 2l0 2" />
              <path d="M12 20l0 2" />
              <path d="M20 12l2 0" />
              <path d="M2 12l2 0" />
            </svg>
          </button>

          <button class="min" id="radar-copy-target-info-${username}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-copy">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M7 9.667a2.667 2.667 0 0 1 2.667 -2.667h8.666a2.667 2.667 0 0 1 2.667 2.667v8.666a2.667 2.667 0 0 1 -2.667 2.667h-8.666a2.667 2.667 0 0 1 -2.667 -2.667l0 -8.666" />
              <path d="M4.012 16.737a2.005 2.005 0 0 1 -1.012 -1.737v-10c0 -1.1 .9 -2 2 -2h10c.75 0 1.158 .385 1.5 1" />
            </svg>
          </button>

          <button class="min" id="radar-remove-target-${username}">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-trash">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M4 7l16 0" />
              <path d="M10 11l0 6" />
              <path d="M14 11l0 6" />
              <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" />
              <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />
            </svg>
          </button>
        </div>
      `;

			this.targetCardsContainer?.appendChild(card);
			this.initializeTargetCard(username);
		});

		openSettingsBtn.addEventListener("click", () => (document.getElementById("radar-settings") as HTMLElement).style.display = "flex");
		closeSettingsBtn.addEventListener("click", () => (document.getElementById("radar-settings") as HTMLElement).style.display = "none");

		removeAllTargetsBtn.addEventListener("click", () => {
			this.targets.forEach((v, n) => {
				const card = document.getElementById(`radar-target-${n}`) as HTMLElement;
				card.remove();
				clearInterval(v.interval);
			});

			this.targets.clear();
		});

		updateFrequency.addEventListener("change", () => {
			this.updateFrequency = updateFrequency.value ? parseInt(updateFrequency.value) : 1500;

			this.targets.forEach((i, n) => {
				clearInterval(i.interval);
				this.setTargetUpdateInterval(n, this.updateFrequency);
			});
		});

		let lx = "";
		let ly = "";
		let lz = "";

		await listen<number[]>("radar:target-info", async (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				const username = buf.readString();
				const uuid = buf.readString();
				const tx = buf.readF64().toFixed(3);
				const ty = buf.readF64().toFixed(3);
				const tz = buf.readF64().toFixed(3);

				(document.getElementById(`radar-target-status-${username}`) as HTMLElement).innerText = "Found";
				(document.getElementById(`radar-target-uuid-${username}`) as HTMLElement).innerText = uuid.substring(0, 12) + "...";
				(document.getElementById(`radar-target-x-${username}`) as HTMLElement).innerText = tx;
				(document.getElementById(`radar-target-y-${username}`) as HTMLElement).innerText = ty;
				(document.getElementById(`radar-target-z-${username}`) as HTMLElement).innerText = tz;

				const target = this.targets.get(username);

				if (target) target.data = { fullUuid: uuid };

				if (lx !== tx || ly !== ty || lz !== tz) {
					const card = document.getElementById(`radar-target-${username}`) as HTMLElement;
					card.classList.add("glow");
					setTimeout(() => card.classList.remove("glow"), 300);
				}

				lx = tx;
				ly = ty;
				lz = tz;

				if ((document.getElementById("radar_chbx_auto-save") as HTMLInputElement).checked) {
					const path = (document.getElementById("radar_option_path") as HTMLInputElement).value;
					const filename = (document.getElementById("radar_option_filename") as HTMLInputElement).value;

					if (await isAbsolute(path)) {
						const buf = new Buffer();
						buf.writeU8(0x09);
						buf.writeString(username);
						buf.writeString(path);
						buf.writeString(filename || "radar_#t");
						buf.writeF64(parseFloat(tx));
						buf.writeF64(parseFloat(ty));
						buf.writeF64(parseFloat(tz));

						const result = await invoke<CommandResult<null>>("send_command", {
							data: buf.toUint8Array(),
						});

						if (result.error) logger.log(`Ошибка сохранения данных радара об игроке ${username}: ${result.error}`, "error");
					}
				}
			} catch (error) {
				logger.log(`Ошибка обработки события "radar:target-info": ${error}`, "error");
			}
		});
	}

	/** Метод активации радара */
	public enable(): void {
		this.active = true;
	}

	/** Метод выключения радара */
	public disable(): void {
		this.active = false;
	}

	/** Метод инициализации карточки цели */
	private initializeTargetCard(username: string): void {
		try {
			const removeTargetBtn = document.getElementById(`radar-remove-target-${username}`) as HTMLButtonElement;
			const copyTargetInfoBtn = document.getElementById(`radar-copy-target-info-${username}`) as HTMLButtonElement;
			const followTargetBtn = document.getElementById(`radar-follow-target-${username}`) as HTMLButtonElement;

			removeTargetBtn.addEventListener("click", () => {
				const card = document.getElementById(`radar-target-${username}`) as HTMLElement;
				card.remove();
				clearInterval(this.targets.get(username)?.interval);
				this.targets.delete(username);
			});

			copyTargetInfoBtn.addEventListener("click", async () => {
				try {
					const status = (document.getElementById(`radar-target-status-${username}`) as HTMLElement).textContent;
					const uuid = this.targets.get(username)?.data?.fullUUID ? this.targets.get(username)?.data.fullUUID : "?";
					const x = (document.getElementById(`radar-target-x-${username}`) as HTMLElement).textContent;
					const y = (document.getElementById(`radar-target-y-${username}`) as HTMLElement).textContent;
					const z = (document.getElementById(`radar-target-z-${username}`) as HTMLElement).textContent;

					const text = `
Никнейм: ${username}
Статус: ${status}
UUID: ${uuid}
Координата X: ${x}
Координата Y: ${y}
Координата Z: ${z}
          `.trim();

					await navigator.clipboard.writeText(text);

					messages.message("Радар", `Данные игрока ${username} успешно скопированы в буфер обмена`);
				} catch (error) {
					logger.log(`Ошибка копирования данных радара: ${error}`, "error");
				}
			});

			followTargetBtn.addEventListener("click", async () => {
				try {
					const xText = document.getElementById(`radar-target-x-${username}`)?.textContent;
					const zText = document.getElementById(`radar-target-z-${username}`)?.textContent;

					if (!xText || !zText) return;

					const x = parseInt(xText);
					const z = parseInt(zText);

					const buf = new Buffer();
					buf.writeU8(0x0A);
					buf.writeString(username);
					buf.writeI32(x);
					buf.writeI32(z);

					const result = await invoke<CommandResult<null>>("send_command", {
						data: buf.toUint8Array(),
					});

					if (result.error) logger.log(`Ошибка преследования ${username}: ${result.error}`, "error");
				} catch (error) {
					logger.log(`Ошибка преследования ${username}: ${error}`, "error");
				}
			});

			this.targets.set(username, { data: null, interval: null });

			this.setTargetUpdateInterval(username, this.updateFrequency);
		} catch (error) {
			logger.log(`Ошибка инициализации цели радара: ${error}`, "error");
		}
	}

	/** Метод установки частоты обновления */
	private setTargetUpdateInterval(username: string, frequency: number) {
		const target = this.targets.get(username);

		if (!target) return;

		target.interval = setInterval(async () => {
			if (!this.active) {
				clearInterval(target.interval);
				return;
			}

			try {
				const buf = new Buffer();
				buf.writeU8(0x08);
				buf.writeU32(2 + username.length);
				buf.writeString(username);

				const result = await invoke<CommandResult<null>>("send_command", {
					data: buf.toUint8Array(),
				});

				if (result.error) {
					logger.log(`Ошибка обновления цели радара ${username}: ${result.error}`, "error");
					clearInterval(target.interval);
					return;
				}
			} catch (error) {
				logger.log(`Ошибка обновления цели радара ${username}: ${error}`, "error");
			}
		}, frequency);
	}
}

const radarModule = new RadarModule();

export { radarModule }
