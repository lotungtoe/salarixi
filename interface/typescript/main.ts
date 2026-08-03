import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Chart, registerables } from "chart.js";

import { plugins } from "./common/structs";
import { logger } from "./utils/logger";
import { messages } from "./utils/message";
import { enableParticles } from "./effects/particles";
import { switchControlWrapper, switchGlobalWrapper } from "./utils/switchers";
import { replenishTriggerRegistry } from "./trigger/replenish";
import { Buffer } from "./buffer";

import { configModule } from "./modules/config";
import { accountModule } from "./modules/account";
import { proxyModule } from "./modules/proxy";
import { chartModule } from "./modules/chart";
import { scriptModule } from "./modules/script";
import { monitoringModule } from "./modules/monitoring";
import { captchaModule } from "./modules/captcha";
import { radarModule } from "./modules/radar";
import { pingModule } from "./modules/ping";
import { translatorModule } from "./modules/translator";
import { pluginModule } from "./modules/plugin";
import { timerModule } from "./modules/timer";
import { docsModule } from "./modules/docs";
import { infoModule } from "./modules/info";
import { updaterModule } from "./modules/updater";
import { stylekitModule } from "./modules/stylekit";
import { scannerModule } from "./modules/scanner";

import { CLIENT_VERSION } from "./version";

Chart.register(...registerables);

Chart.defaults.font.family = "'Inter', sans-serif";
Chart.defaults.font.size = 13;
Chart.defaults.font.weight = "normal";

let stopping = false;
let botsWereFullyConnected = false;
let totalBots = 0;
let connectedBots = 0;

export let processActivity: boolean = false;
export let quickTasksAllowed: boolean = true;

const pressedKeys: { [x: string]: boolean } = {
	alt: false,
	shift: false,
	f: false,
	i: false,
	c: false,
	j: false,
	w: false,
	a: false,
	s: false,
	d: false,
	q: false,
	h: false,
	u: false,
	l: false,
	p: false,
	g: false,
	r: false,
	t: false,
	z: false
};

export let globalWrappers: Array<{ id: string; el: HTMLElement }> = [];
export let controlWrappers: Array<{ id: string; el: HTMLElement }> = [];
export let latestControlWrapper: HTMLElement | null = null;

/** Функция обновления отображаемого статуса процесса */
function updateProcessStatus(): void {
	const processStatus = document.getElementById("process-status");
	if (!processStatus) return;

	if (stopping) {
		processStatus.innerText = `stopping ${connectedBots} bots...`;
		processStatus.style.color = "var(--process-stopping-color)";
		return;
	}

	if (processActivity) {
		if (!botsWereFullyConnected && connectedBots === totalBots) {
			botsWereFullyConnected = true;
		}

		if (botsWereFullyConnected) {
			processStatus.innerText = `active (${connectedBots}/${totalBots} bots)`;
			processStatus.style.color = "var(--process-active-color)";
		} else {
			processStatus.innerText = `launching (${connectedBots}/${totalBots} bots)...`;
			processStatus.style.color = "var(--process-launching-color)";
		}
	} else {
		processStatus.innerText = "inactive";
		processStatus.style.color = "var(--process-inactive-color)";
	}
}

/** Функция изменения значения `quickTasksAllowed` */
export function setQuickTasksAllowed(value: boolean): void {
	quickTasksAllowed = value;
}

/** Функция запуска ботов */
async function startBots(): Promise<void> {
	try {
		const options: {
			basic: any;
			accounts: any;
			plugins: any;
			captcha_bypass: any;
			webhook: any;
		} = {
			basic: {},
			accounts: {},
			plugins: {},
			captcha_bypass: {},
			webhook: {}
		};

		document.querySelectorAll<HTMLElement>("[launch-option]").forEach(o => {
			const optionSection = o.getAttribute("section") || "basic";
			const optionKey = o.getAttribute("key") || "";

			const current = options[optionSection as "basic" | "plugins" | "captcha_bypass" | "webhook"];

			if (o.tagName.toLocaleLowerCase() === "input") {
				const input = o as HTMLInputElement;

				if (input.type === "checkbox") {
					current[optionKey] = input.checked;
				} else {
					const defaultValue = input.getAttribute("default");

					if (input.type === "number") {
						current[optionKey] = input.value ? parseInt(input.value) : defaultValue ? parseInt(defaultValue!) : null;
					} else {
						current[optionKey] = input.value || defaultValue;
					}
				}
			} else if (o.tagName.toLocaleLowerCase() === "select") {
				const select = o as HTMLSelectElement;
				current[optionKey] = select.value;
			} else if (o.tagName.toLocaleLowerCase() === "textarea") {
				const textarea = o as HTMLTextAreaElement;
				current[optionKey] = textarea.value;
			}
		});

		options.accounts = accountModule.getSelectedAccounts();
		options.basic["script"] = options.basic.use_auto_script ? (document.getElementById("user-script") as HTMLTextAreaElement).value : null;

		let currentLaunchCount = 0;

		if (options.basic.use_accounts) {
			for (const _ in options.accounts) currentLaunchCount++;
		} else {
			currentLaunchCount = options.basic.bots_count as number;
		}

		if (currentLaunchCount > 255 || currentLaunchCount < 1) {
			messages.message("Cистема", "Невозможно запустить ботов");
			logger.log("Невозможно запустить ботов: Number of simultaneously launching bots cannot be less than 1 and more than 255", "warning");
			return;
		}

		totalBots = currentLaunchCount;

		for (const name in plugins) options.plugins[name.replaceAll("-", "_")] = plugins[name].enable;

		const text = `Запуск ботов на сервер ${options.basic.address}...`;

		logger.log(text, "info");
		messages.message("Cистема", text);

		const optionsStr = JSON.stringify(options);
		const bytes = new TextEncoder().encode(optionsStr);

		const success = await invoke<boolean>("launch_bots", { options: bytes });
		if (!success) return;

		processActivity = true;
		updateProcessStatus();

		logger.log("Включение рантайм модулей...", "system");

		monitoringModule.maxChatHistoryLength = parseInt((document.getElementById("monitoring_option_chat-history-length") as HTMLInputElement).value || "50");

		monitoringModule.enable(parseInt((document.getElementById("monitoring_option_update-frequency") as HTMLInputElement).value || "1800"));
		monitoringModule.wait();

		if (options.basic.use_anti_captcha) captchaModule.enable(options.captcha_bypass.captcha_type, options.captcha_bypass.solve_mode);

		radarModule.enable();

		logger.log("Рантайм модули включены", "system");
	} catch (error) {
		logger.log(`Ошибка запуска ботов: ${error}`, "error");
	}
}

/** Функция остановки ботов */
async function stopBots(): Promise<void> {
	if (stopping) return;

	logger.log("Остановка ботов...", "info");

	stopping = true;
	updateProcessStatus();

	try {
		const success = await invoke<boolean>("stop_bots");

		if (!success) {
			updateProcessStatus();
			return;
		}

		processActivity = false;
		botsWereFullyConnected = false;
		totalBots = 0;
		connectedBots = 0;
		stopping = false;

		updateProcessStatus();

		logger.log("Выключение рантайм модулей...", "system");

		monitoringModule.disable();
		captchaModule.disable();
		radarModule.disable();

		logger.log("Рантайм модули выключены", "system");
	} catch (error) {
		logger.log(`Ошибка остановки ботов: ${error}`, "error");
	}
}

/** Функция обновления состояния указанного модуля управления */
async function updateModuleState(index: number, state: number): Promise<void> {
	try {
		const buf = new Buffer();
		buf.writeU8(index);

		const group = (document.getElementById("control-group") as HTMLInputElement).value.replace(" ", "");
		buf.writeString(group !== "" ? group : "global");

		const elements = document.querySelectorAll(`[module-index="${index}"]`);
		let filteredElements: Map<number, HTMLSelectElement | HTMLInputElement> = new Map();

		elements.forEach(e => {
			const tag = e.tagName.toLowerCase();
			if (tag === "button") return;

			const ordinal = e.getAttribute("ordinal");
			if (!ordinal) return;

			filteredElements.set(parseInt(ordinal), e as any);
		});

		let orderedElements: Array<HTMLSelectElement | HTMLInputElement> = [];

		for (let i = 0; i < filteredElements.size; i++) {
			const el = filteredElements.get(i);
			if (!el) continue;
			orderedElements.push(el);
		}

		orderedElements.forEach(e => {
			const tag = e.tagName.toLowerCase();

			if (tag === "select") {
				// Пусть u8, пока нету селекта, который хранит больше 255 опций
				buf.writeU8((e as HTMLSelectElement).selectedIndex);
			} else {
				const input = e as HTMLInputElement;

				if (e.type === "checkbox") {
					buf.writeU8(input.checked ? 1 : 0);
				} else {
					const optional = input.getAttribute("optional") === "true";

					if (optional) {
						if (input.value === "") {
							buf.writeU8(0);
							return;
						} else {
							buf.writeU8(1);
						}
					}

					if (e.type === "number") {
						const ty = e.getAttribute("num-type");
						if (!ty) return;

						const value = Number(input.value);

						switch (ty) {
							case "u8":
								buf.writeU8(value);
								break;
							case "u16":
								buf.writeU16(value);
								break;
							case "u64":
								buf.writeU64(BigInt(value));
								break;
							case "i32":
								buf.writeI32(value);
								break;
							case "i64":
								buf.writeI64(BigInt(value));
								break;
							case "f32":
								buf.writeF32(value);
								break;
							case "f64":
								buf.writeF64(value);
								break;
						}
					} else {
						buf.writeString(input.value);
					}
				}
			}
		});

		// Пока так, состояние это последняя опция, иначе логически не поставить
		buf.writeU8(state);

		await invoke("control_bots", { data: buf.toUint8Array() });
	} catch (error) {
		logger.log(`Ошибка изменения состояния модуля (module-index=${index}): ${error}`, "error");
	}
}

/** Функция инициализации глобальных элементов */
function initGlobalElements(): void {
	const startBotsProcessBtn = document.getElementById("start") as HTMLButtonElement;
	const stopBotsProcessBtn = document.getElementById("stop") as HTMLButtonElement;
	const setRandomValuesBtn = document.getElementById("random") as HTMLButtonElement;
	const clearInputValuesBtn = document.getElementById("clear") as HTMLButtonElement;

	const dashboardBtns = document.querySelectorAll<HTMLButtonElement>(".dashboard button");
	const controlBtns = document.querySelectorAll<HTMLButtonElement>(".control-dashboard .button-list button");

	dashboardBtns.forEach(btn => {
		if (btn.id === "main") btn.classList.add("selected");
		btn.addEventListener("click", () => {
			const path = btn.getAttribute("path");
			if (!path) return;
			switchGlobalWrapper(path);
			dashboardBtns.forEach(b => b.classList.remove("selected"));
			btn.classList.add("selected");
		});
	});

	controlBtns.forEach(btn => {
		if (btn.id === "control-chat") btn.classList.add("selected");
		btn.addEventListener("click", () => {
			const path = btn.getAttribute("path");
			if (!path) return;
			switchControlWrapper(path);
			controlBtns.forEach(b => b.classList.remove("selected"));
			btn.classList.add("selected");
			latestControlWrapper = document.getElementById(path);
		});
	});

	document.querySelectorAll<HTMLButtonElement>('[module-toggler]').forEach(e => e.addEventListener("click", async () => await updateModuleState(Number(e.getAttribute("module-index")), Number(e.getAttribute("state")))));

	document.addEventListener("keydown", async (e) => {
		if (!processActivity || !quickTasksAllowed) return;
		const key = e.key.toLowerCase();
		for (const k in pressedKeys) key === k ? pressedKeys[key] = true : null;
		if (pressedKeys.shift && pressedKeys.i && pressedKeys.c) {
			await invoke("quick_task", { name: "clear-inventory" });
		} else if (pressedKeys.shift && pressedKeys.f && pressedKeys.c) {
			await invoke("quick_task", { name: "form-circle" });
		} else if (pressedKeys.shift && pressedKeys.f && pressedKeys.l) {
			await invoke("quick_task", { name: "form-line" });
		} else if (pressedKeys.shift && pressedKeys.p && pressedKeys.s) {
			await invoke("quick_task", { name: "pathfinder-stop" });
		} else if (pressedKeys.shift && pressedKeys.g && pressedKeys.f) {
			await invoke("quick_task", { name: "fly" });
		} else if (pressedKeys.shift && pressedKeys.g && pressedKeys.r) {
			await invoke("quick_task", { name: "rise" });
		} else if (pressedKeys.shift && pressedKeys.g && pressedKeys.c) {
			await invoke("quick_task", { name: "capsule" });
		} else if (pressedKeys.alt && pressedKeys.shift && pressedKeys.q) {
			await invoke("quick_task", { name: "quit" });
		} else if (pressedKeys.shift && pressedKeys.w) {
			await invoke("quick_task", { name: "move-forward" });
		} else if (pressedKeys.shift && pressedKeys.s) {
			await invoke("quick_task", { name: "move-backward" });
		} else if (pressedKeys.shift && pressedKeys.a) {
			await invoke("quick_task", { name: "move-left" });
		} else if (pressedKeys.shift && pressedKeys.d) {
			await invoke("quick_task", { name: "move-right" });
		} else if (pressedKeys.shift && pressedKeys.j) {
			await invoke("quick_task", { name: "jump" });
		} else if (pressedKeys.shift && pressedKeys.c) {
			await invoke("quick_task", { name: "shift" });
		} else if (pressedKeys.shift && pressedKeys.u) {
			await invoke("quick_task", { name: "unite" });
		} else if (pressedKeys.shift && pressedKeys.t) {
			await invoke("quick_task", { name: "turn" });
		} else if (pressedKeys.shift && pressedKeys.z) {
			await invoke("quick_task", { name: "zero" });
		}
	});

	document.addEventListener("keyup", (e) => {
		if (!processActivity) return;
		const key = e.key.toLowerCase();
		for (const k in pressedKeys) key === k ? pressedKeys[key] = false : null;
	});

	startBotsProcessBtn.addEventListener("click", async () => await startBots());
	stopBotsProcessBtn.addEventListener("click", async () => await stopBots());

	setRandomValuesBtn.addEventListener("click", () => {
		const gen = (e: string): void => {
			let current = document.getElementById(e) as HTMLInputElement;
			switch (e) {
				case "settings_option_bots-count":
					const randomQuantity = Math.floor(Math.random() * (50 - 10 + 1) + 10);
					current.valueAsNumber = randomQuantity; break;
				case "settings_option_join-delay":
					const randomDelay = Math.floor(Math.random() * (7000 - 1000 + 1) + 1000);
					current.valueAsNumber = randomDelay; break;
			}
		}

		gen("settings_option_bots-count");
		gen("settings_option_join-delay");
	});

	clearInputValuesBtn.addEventListener("click", () => {
		(document.getElementById("settings_option_address") as HTMLInputElement).value = "";
		(document.getElementById("settings_option_bots-count") as HTMLInputElement).value = "";
		(document.getElementById("settings_option_join-delay") as HTMLInputElement).value = "";
	});

	const clientSettingsContainer = document.querySelector<HTMLDivElement>(".client-settings-wrapper");
	if (!clientSettingsContainer) return;

	document.getElementById("open-client-settings")?.addEventListener("click", () => clientSettingsContainer.style.display = "flex");
	document.getElementById("close-client-settings")?.addEventListener("click", () => clientSettingsContainer.style.display = "none");
}

/** Функция добавления открывающейся ссылки для определённого события элемента */
export function addOpeningUrlTo(id: string, event: string, url: string): void {
	const el = document.getElementById(id);
	if (!el) return;
	el.addEventListener(event, async () => {
		try {
			await invoke("open_url", { url: url });
		} catch (error) {
			logger.log(`Ошибка открытия URL: ${error}`, "error");
		}
	});
}

/** Вспомогательная функция инициализации титул бара */
function initTitleBar(): void {
	(document.getElementById("title-version") as HTMLElement).innerText = `v${CLIENT_VERSION}`;
	(document.getElementById("window-minimize") as HTMLButtonElement).addEventListener("click", async () => await getCurrentWindow().minimize());
	(document.getElementById("window-close") as HTMLButtonElement).addEventListener("click", async () => {
		await configModule.saveCurrentConfig();
		await invoke("exit");
	});
}

/** Функция инициализации слушателей основных событий */
async function listenEvents(): Promise<void> {
	await listen<{ text: string; class: string; }>("system:log", (event) => {
		try {
			logger.log(event.payload.text, event.payload.class);
		} catch (error) {
			logger.log(`Ошибка принятие события "system:log": ${error}`, "error");
		}
	});

	await listen<{ name: string; content: string; }>("system:message", (event) => {
		try {
			messages.message(event.payload.name, event.payload.content);
		} catch (error) {
			logger.log(`Ошибка принятие события "system:message": ${error}`, "error");
		}
	});

	await listen<number>("system:connected-bots", (event) => {
		try {
			connectedBots = event.payload;
			updateProcessStatus();
		} catch (error) {
			logger.log(`Ошибка принятие события "system:connected-bots": ${error}`, "error");
		}
	});
}

document.addEventListener("DOMContentLoaded", () => {
	initTitleBar();

	document.addEventListener("contextmenu", e => e.preventDefault());

	logger.init();
	messages.init();

	logger.log(`Клиент запущен, версия ${CLIENT_VERSION}`, "info");

	try {
		timerModule.init();

		initGlobalElements();
		replenishTriggerRegistry();

		document.querySelectorAll('[global="true"]').forEach(c => globalWrappers.push({ id: c.id, el: c as HTMLDivElement }));
		document.querySelectorAll('[sector="true"]').forEach(c => controlWrappers.push({ id: c.id, el: c as HTMLDivElement }));

		enableParticles();

		chartModule.init();
		radarModule.init();
		scriptModule.init();
		pingModule.init();
		stylekitModule.init();
	} catch (error) {
		logger.log(`Ошибка инициализации (sync): ${error}`, "error");
	}
});

document.addEventListener("DOMContentLoaded", async () => {
	try {
		await listenEvents();

		await configModule.init();

		scriptModule.updateLineCounter();

		await accountModule.init();
		await monitoringModule.init();
		await captchaModule.init();
		await proxyModule.init();
		await scannerModule.init();
		await translatorModule.init();
		await pluginModule.init();
		await docsModule.init();
		await infoModule.init();

		const selectDiscordRpcMode = document.getElementById("interface_select_discord-rpc") as HTMLSelectElement;

		selectDiscordRpcMode.addEventListener("change", async () => await invoke("set_discord_rpc", { version: CLIENT_VERSION, state: selectDiscordRpcMode.value === "enable" }));
		await invoke("set_discord_rpc", { version: CLIENT_VERSION, state: selectDiscordRpcMode.value === "enable" });

		updaterModule.init();
	} catch (error) {
		logger.log(`Ошибка инициализации (async): ${error}`, "error");
	}
});
