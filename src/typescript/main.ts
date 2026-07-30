import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Chart, registerables } from "chart.js";

import { plugins } from "./common/structs";
import { logger } from "./utils/logger";
import { messages } from "./utils/message";
import { enableParticles } from "./effects/particles";
import { switchControlWrapper, switchGlobalWrapper } from "./utils/switchers";

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
import { replenishTriggerRegistry } from "./trigger/replenish";

Chart.register(...registerables);

Chart.defaults.font.family = "'Inter', sans-serif";
Chart.defaults.font.size = 13;
Chart.defaults.font.weight = "normal";

export const currentClientVersion = "1.1.1";

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

    const success = await invoke("launch_bots", { options: bytes }) as boolean;
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
    const success = await invoke("stop_bots") as boolean;

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

/** Функция отправки данных в поток Rust для модулей управления */
async function controlBots(name: string, state: number): Promise<void> {
  try {
    const elements = document.querySelectorAll(`[control="${name}"]`);

    let options: Record<string, any> = {};

    elements.forEach(e => e.tagName.toLowerCase() === "select" ? options[(e as HTMLSelectElement).name] = (e as HTMLSelectElement).selectedIndex : (e as HTMLInputElement).type === "checkbox" ? options[(e as HTMLInputElement).name] = (e as HTMLInputElement).checked : options[(e as HTMLInputElement).name] = (e as HTMLInputElement).type === "number" ? Number((e as HTMLInputElement).value) : (e as HTMLInputElement).value);

    const group = (document.getElementById("control-group") as HTMLInputElement).value.replace(" ", "");

    await invoke("control_bots", {
      name: name,
      options: {
        ...options,
        state: state
      },
      group: group !== "" ? group : "global"
    });
  } catch (error) {
    logger.log(`Ошибка управления (${name}): ${error}`, "error");
  }
}

/** Функция инициализации глобальных элементов */
function initGlobalElements(): void {
  const startBotsProcessBtn = document.getElementById("start") as HTMLButtonElement;
  const stopBotsProcessBtn = document.getElementById("stop") as HTMLButtonElement;
  const setRandomValuesBtn =  document.getElementById("random") as HTMLButtonElement;
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

  document.querySelectorAll<HTMLButtonElement>('[control-toggler="true"]').forEach(e => e.addEventListener("click", async () => await controlBots(e.name, Number(e.getAttribute("state")))));

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
  (document.getElementById("title-version") as HTMLElement).innerText = `v${currentClientVersion}`;
  (document.getElementById("window-minimize") as HTMLButtonElement).addEventListener("click", async () => await getCurrentWindow().minimize());
  (document.getElementById("window-close") as HTMLButtonElement).addEventListener("click", async () => {
    await configModule.saveCurrentConfig();
    await invoke("exit");
  });
}

/** Функция инициализации слушателей основных событий */
async function listenEvents(): Promise<void> {
  await listen("log", (event) => {
    try {
      const payload = event.payload as { text: string; class: string; };
      logger.log(payload.text, payload.class);
    } catch (error) {
      logger.log(`Ошибка принятие события "log": ${error}`, "error");
    }
  });

  await listen("message", (event) => {
    try {
      const payload = event.payload as { name: string; content: string; };
      messages.message(payload.name, payload.content);
    } catch (error) {
      logger.log(`Ошибка принятие события "message": ${error}`, "error");
    }
  });

	await listen("connected-bots-count", (event) => {
    try {
      const payload = event.payload as { count: number; };
			connectedBots = payload.count;
			updateProcessStatus();
    } catch (error) {
      logger.log(`Ошибка принятие события "connected-bots-count": ${error}`, "error");
    }
  });
}

document.addEventListener("DOMContentLoaded", () => {
  initTitleBar();

  document.addEventListener("contextmenu", e => e.preventDefault());

  logger.init();
  messages.init();

  logger.log(`Клиент запущен, версия ${currentClientVersion}`, "info");

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
    await translatorModule.init();
    await pluginModule.init();
    await docsModule.init();
    await infoModule.init();

    const selectDiscordRpcMode = document.getElementById("interface_select_discord-rpc") as HTMLSelectElement;

    selectDiscordRpcMode.addEventListener("change", async () => await invoke("set_discord_rpc", { version: currentClientVersion, state: selectDiscordRpcMode.value === "enable" }));
    await invoke("set_discord_rpc", { version: currentClientVersion, state: selectDiscordRpcMode.value === "enable" });

    updaterModule.init();
  } catch (error) {
    logger.log(`Ошибка инициализации (async): ${error}`, "error");
  }
});
