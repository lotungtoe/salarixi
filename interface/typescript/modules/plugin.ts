import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { PluginOption, plugins, PluginValueType, pluginValueTypeToInputType } from "../common/structs";
import { processActivity } from "../main";
import { loadCacheOrNew, LcfResult, LnfResult, loadCache, saveCache, TimeType } from "../utils/cache";

/** Модуль загрузки и управления плагинами */
class PluginModule {
	private enabledPlugins: Array<string> = [];

	/** Метод инициализации функций, связанных с плагинами */
	public async init(): Promise<void> {
		const initres = await invoke<CommandResult<null>>("initialize_plugins_dir");

		if (initres.error) {
			logger.log(`Ошибка инициализации плагинов: ${initres.error}`, "error");
			return;
		}

		await this.loadSavedPluginsSettings();

		this.createAndInitPlugins();

		await this.loadSavedPluginsStates();
		await this.initPluginsInfo();
	}

	/** Метод обновления сохранения состояния плагина */
	private async updatePluginState(id: string, enabled: boolean): Promise<void> {
		if (processActivity) {
			logger.log(`Не удалось изменить состояние плагина ${id}: plugin states can only be changed before startup`, "warning");
			return;
		}

		try {
			enabled ? this.enabledPlugins.push(id) : delete this.enabledPlugins[this.enabledPlugins.indexOf(id)];
			plugins[id].enabled = enabled;

			const toggler = document.getElementById(`${id}-toggler`) as HTMLButtonElement;

			if (enabled) {
				toggler.setAttribute("state", "false");
				toggler.style.color = "#4ed618";
				toggler.innerHTML = `
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-check">
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M5 12l5 5l10 -10" />
          </svg>
        `;
			} else {
				toggler.setAttribute("state", "true");
				toggler.style.color = "#e61919";
				toggler.innerHTML = `
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M18 6l-12 12" />
            <path d="M6 6l12 12" />
          </svg>
        `;
			}

			if (this.enabledPlugins.length < 1) {
				await invoke<CommandResult<null>>("s_remove_file", { path: "plugins/enabled.json" });
			} else {
				const str = JSON.stringify(this.enabledPlugins, null, 2);
				const enc = new TextEncoder();
				const ecd = enc.encode(str);

				const result = await invoke<CommandResult<null>>("s_save_file", {
					path: "plugins/enabled.json",
					data: ecd,
				});

				if (result.error) logger.log(`Ошибка сохранения состояния плагина ${id}: ${result.error}`, "error");
			}
		} catch (error) {
			logger.log(`Ошибка сохранения состояния плагина ${id}: ${error}`, "error");
		}
	}

	/** Метод загрузки сохранённых состояний плагинов */
	private async loadSavedPluginsStates(): Promise<void> {
		try {
			const result = await invoke<CommandResult<number[]>>("s_read_file", { path: "plugins/enabled.json" });

			if (result.error) {
				logger.log(`Ошибка загрузки контента из файла "plugins/enabled.json": ${result.error}`, "error");
				return;
			}

			if (!result.data || result.data.length < 1) return;

			const u8a = new Uint8Array(result.data);
			const dec = new TextDecoder("utf-8");
			const dcd = dec.decode(u8a);
			const parsed = JSON.parse(dcd);

			for (const id of parsed) {
				const plugin = plugins[id];
				if (!plugin) continue;

				const toggler = document.getElementById(`${id}-toggler`) as HTMLButtonElement;

				toggler.setAttribute("state", "false");
				toggler.style.color = "#4ed618";
				toggler.innerHTML = `
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-check">
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M5 12l5 5l10 -10" />
          </svg>
        `;
			}

			logger.log("Состояния плагинов успешно загружены и применены", "system");
		} catch (error) {
			logger.log(`Ошибка загрузки сохранённых состояний плагинов: ${error}`, "error");
		}
	}

	/** Метод создания и инициализации карточек плагинов */
	private createAndInitPlugins(): void {
		for (const id in plugins) this.createPlugin(id);

		document.querySelectorAll<HTMLButtonElement>('[plugin-toggler="true"]').forEach(t => t.addEventListener("click", async () => {
			const id = t.getAttribute("plugin-id");
			if (!id) return;
			const state = t.getAttribute("state") === "true";
			await this.updatePluginState(id, state);
		}));

		const chv = (b: HTMLButtonElement, v: string) => {
			const path = b.getAttribute("path");
			if (!path) return;
			(document.getElementById(path) as HTMLElement).style.display = v;
		};

		document.querySelectorAll<HTMLButtonElement>('[plugin-od="true"]').forEach(b => b.addEventListener("click", () => chv(b, "flex")));
		document.querySelectorAll<HTMLButtonElement>('[plugin-cd="true"]').forEach(b => b.addEventListener("click", () => chv(b, "none")));
		document.querySelectorAll<HTMLButtonElement>('[plugin-os="true"]').forEach(b => b.addEventListener("click", () => chv(b, "flex")));
		document.querySelectorAll<HTMLButtonElement>('[plugin-cs="true"]').forEach(b => b.addEventListener("click", () => chv(b, "none")));
	}

	/** Метод создания плагина */
	private createPlugin(id: string): void {
		try {
			const obj = plugins[id];
			this.createPluginCard(id, obj.name);
			this.createPluginDescription(id, obj.name);
			this.createPluginSettings(id, obj.name, obj.settings);
		} catch (error) {
			logger.log(`Ошибка инициализации плагина ${id}: ${error}`, "error");
		}
	}

	/** Метод создания карточки плагина в списке */
	private createPluginCard(id: string, name: string): void {
		const card = document.createElement("div");
		card.className = "plugin-card";
		card.id = `${id}-plugin`;

		card.innerHTML = `
      <label class="name" id="${id}-name">${name}</label>

      <div class="controls">
        <button class="min" plugin-od="true" path="${id}-plugin-description">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-align-justified">
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M4 6l16 0" />
            <path d="M4 12l16 0" />
            <path d="M4 18l12 0" />
          </svg>
        </button>

        <button class="min" plugin-os="true" path="${id}-plugin-settings">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-settings">
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M10.325 4.317c.426 -1.756 2.924 -1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543 -.94 3.31 .826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756 .426 1.756 2.924 0 3.35a1.724 1.724 0 0 0 -1.066 2.573c.94 1.543 -.826 3.31 -2.37 2.37a1.724 1.724 0 0 0 -2.572 1.065c-.426 1.756 -2.924 1.756 -3.35 0a1.724 1.724 0 0 0 -2.573 -1.066c-1.543 .94 -3.31 -.826 -2.37 -2.37a1.724 1.724 0 0 0 -1.065 -2.572c-1.756 -.426 -1.756 -2.924 0 -3.35a1.724 1.724 0 0 0 1.066 -2.573c-.94 -1.543 .826 -3.31 2.37 -2.37c1 .608 2.296 .07 2.572 -1.065" />
            <path d="M9 12a3 3 0 1 0 6 0a3 3 0 0 0 -6 0" />
          </svg>
        </button>

        <button class="min" plugin-toggler="true" plugin-id="${id}" state="true" id="${id}-toggler" style="color: #d61818;">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M18 6l-12 12" />
            <path d="M6 6l12 12" />
          </svg>
        </button>
      </div>
    `;

		document.getElementById("plugin-list")?.appendChild(card);
	}

	/** Метод создания контейнера с описанием плагина */
	private createPluginDescription(id: string, name: string): void {
		const description = document.createElement("div");
		description.className = "cover";
		description.id = `${id}-plugin-description`;

		description.innerHTML = `
      <div class="panel with-header">
        <div class="left">
          <div class="header">Описание плагина "${name}"</div>
        </div>

        <div class="right">
          <button class="min" plugin-cd="true" path="${id}-plugin-description">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M18 6l-12 12" />
              <path d="M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <div class="plugin-description" id="${id}-description"></div>
      <p class="plugin-latest-update">Последнее обновление плагина: <span id="${id}-latest-update">?</span></p>
    `;

		document.getElementById("plugins-container")?.appendChild(description);
	}

	/** Метод создания контейнера с настройками плагина */
	private createPluginSettings(id: string, name: string, options: Array<PluginOption>): void {
		const container = document.createElement("div");
		container.className = "cover";
		container.id = `${id}-plugin-settings`;

		container.innerHTML = `
      <div class="panel with-header">
        <div class="left">
          <div class="header">Настройки плагина "${name}"</div>
        </div>

        <div class="right">
          <button class="min" plugin-cs="true" path="${id}-plugin-settings">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
              <path stroke="none" d="M0 0h24v24H0z" fill="none" />
              <path d="M18 6l-12 12" />
              <path d="M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    `;

		const settings = document.createElement("div");
		settings.className = "plugin-settings";

		for (const option of options) {
			const e = document.createElement("div");
			e.className = "pretty-input-wrapper";

			const signature = document.createElement("p");
			signature.className = "signature fix";
			signature.innerText = option["key"];

			const input = document.createElement("input");
			input.type = pluginValueTypeToInputType(option["vtype"]);
			input.placeholder = option["placeholder"];

			input.value = typeof option["value"] === "string" ? option["value"] : String(option["value"]);
			input.addEventListener("input", async () => {
				if (input.value === "") {
					option["value"] = option["default"];
					return;
				}

				switch (option["vtype"]) {
					case PluginValueType.U32:
					case PluginValueType.F32:
					case PluginValueType.F64:
						option["value"] = Number(input.value);
						break;
					case PluginValueType.Bool:
						option["value"] = input.value === "true" || input.value === "false" ? input.value === "true" : option["default"];
						break;
					case PluginValueType.String:
						option["value"] = input.value;
						break;
				}

				await this.savePluginsSettings();
			});

			e.appendChild(signature);
			e.appendChild(input);

			settings.appendChild(e);
		}

		container.appendChild(settings);

		document.getElementById("plugins-container")?.appendChild(container);
	}

	/** Метод загрузки сохранённых настроек плагинов */
	private async loadSavedPluginsSettings(): Promise<void> {
		try {
			const result = await invoke<CommandResult<number[]>>("s_read_file", { path: "plugins/settings.json" });

			if (result.error) {
				logger.log(`Ошибка загрузки контента из файла "plugins/settings.json": ${result.error}`, "error");
				return;
			}

			if (!result.data || result.data.length < 1) return;

			const u8a = new Uint8Array(result.data);
			const dec = new TextDecoder("utf-8");
			const dcd = dec.decode(u8a);
			const parsed = JSON.parse(dcd);

			for (const id in parsed) {
				const plugin = plugins[id];
				if (!plugin) continue;

				const opts = parsed[id];

				for (const key in opts) {
					const value = opts[key];
					plugin.settings[key as any] = value;
				}
			}

			logger.log("Настройки плагинов успешно загружены и применены", "system");
		} catch (error) {
			logger.log(`Ошибка сохранения настроек плагинов: ${error}`, "error");
		}
	}

	/** Метод сохранения настроек плагинов в хранилище */
	private async savePluginsSettings(): Promise<void> {
		try {
			let transformed_obj: Record<string, Record<string, any>> = {};

			for (const id in plugins) {
				const plugin = plugins[id];
				const settings = plugin.settings;
				const transformed_opts: Record<string, any> = {};

				for (const st of settings) transformed_opts[st.key] = st.value;

				transformed_obj[id] = transformed_opts;
			}

			const str = JSON.stringify(transformed_obj, null, 2);
			const enc = new TextEncoder();
			const ecd = enc.encode(str);

			const result = await invoke<CommandResult<null>>("s_save_file", {
				path: "plugins/settings.json",
				data: ecd,
			});

			if (result.error) {
				logger.log(`Ошибка сохранения настроек плагинов: ${result.error}`, "error");
				return;
			}
		} catch (error) {
			logger.log(`Ошибка сохранения настроек плагинов: ${error}`, "error");
		}
	}

	/** Метод инициализации информации о плагинах */
	private async initPluginsInfo(): Promise<void> {
		try {
			logger.log("Загрузка информации о плагинах...", "system");

			const onerr = (err: string) => logger.log(`Ошибка загрузки информации о плагинах (не критично): ${err}`, "nc-error");
			const result = await loadCacheOrNew<any>(this.lcf, this.lnf, this.scf, onerr);
			if (!result.data) return;

			const data = result.data;
			const pluginList = data["list"] ?? data["array"] ?? data["map"] ?? data;

			// Может быть такое что названия полей изменились, или вовсе вся структура
			// данных обновлена, поэтому здесь стоит делать ещё одну проверку
			if (!pluginList) {
				logger.log("Ошибка загрузки информации о плагинах (не критично): information is corrupted or missing", "nc-error");
				return;
			}

			await this.extractPluginsInfo(pluginList);

			logger.log(`Информация о плагинах загружена (размер ${result.size})`, "system");
		} catch (error) {
			logger.log(`Ошибка загрузки информации о плагинах (не критично): ${error}`, "nc-error");
		}
	}

	/** Метод извлечения информации плагинов */
	private async extractPluginsInfo(pluginList: any): Promise<void> {
		const isDevMode = await invoke<boolean>("is_dev_mode");

		for (const plugin of pluginList) {
			const id = plugin["id"] ?? plugin["name"] ?? plugin["header"] ?? plugin["kind"];
			const desc = plugin["description"] ?? plugin["purpose"];
			const updated = plugin["latest_update"] ?? plugin["updated"] ?? plugin["modified"];

			if (!plugins[id]?.modified) continue;

			if (!isDevMode && updated !== plugins[id].modified) {
				document.getElementById(`${id}-plugin`)?.classList.add("deprecated");

				const tag = document.createElement("span");
				tag.className = "tag";
				tag.innerText = "deprecated";

				document.getElementById(`${id}-name`)?.appendChild(tag);
			}

			const pluginDescriptionContainer = document.getElementById(`${id}-description`) as HTMLElement;
			const pluginLatestUpdateContainer = document.getElementById(`${id}-latest-update`) as HTMLElement;

			for (const p of desc) {
				const el = document.createElement("p");
				el.innerText = p;
				pluginDescriptionContainer.appendChild(el);
			}

			pluginLatestUpdateContainer.innerText = updated;
		}
	}

	/** Метод загрузки актуальной информации о плагинах */
	private async lnf(): Promise<LnfResult> {
		try {
			const plugins = await invoke<any>("download_json", {
				url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.plugins.json"
			});

			return { data: plugins, error: plugins ? null : "resource moved or unavailable" };
		} catch (err) {
			return { data: null, error: String(err) };
		}
	}

	/** Метод загрузки кэшированной информации о плагинах */
	private async lcf(): Promise<LcfResult> {
		const result = await loadCache("plugins-info", "plugins-info.json", 4, TimeType.Day);
		if (!result.data) return { data: null, incompatible: false, expired: false };

		return {
			data: JSON.parse(result.data),
			incompatible: result.error === "incompatible",
			expired: result.error === "expired",
		};
	}

	/** Метод кэширования информации о плагинах */
	private async scf(info: string): Promise<void> {
		const result = await saveCache(JSON.stringify(info, null, 2), "plugins-info.json");

		if (!result.data && result.error) {
			logger.log(`Ошибка кэширования информации о плагинах (не критично): ${result.error}`, "nc-error");
		}
	}
}

const pluginModule = new PluginModule();

export { pluginModule }
