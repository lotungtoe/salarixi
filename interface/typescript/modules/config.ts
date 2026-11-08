import { logger } from "../utils/logger";
import { messages } from "../utils/message";
import { open } from "@tauri-apps/plugin-dialog";
import { triggerRegistry } from "../trigger/registry";
import { invoke } from "@tauri-apps/api/core";

/** Тип значения поля конфигурации */
type ConfigValue = string | number | boolean | null;

/** Модуль управления пользовательской конфигурацией */
class ConfigModule {
	private configList: HTMLDivElement | null = null;
	private configs: Map<string, boolean> = new Map();

	/** Метод инициализации конфигуратора */
	public async init(): Promise<void> {
		const initres = await invoke<CommandResult<null>>("initialize_configs_dir");

		if (initres.error) {
			logger.log(`Ошибка инициализации конфигуратора: ${initres.error}`, "error");
			return;
		}

		this.configList = document.getElementById("config-list") as HTMLDivElement | null;

		const configSearchQuery = document.getElementById("config-search-query") as HTMLInputElement;
		configSearchQuery.addEventListener("input", () => this.searchConfigs(configSearchQuery.value));

		document.getElementById("import-configs")?.addEventListener("click", async () => await this.importConfigs());
		document.getElementById("export-current-config")?.addEventListener("click", async () => await this.exportCurrentConfig());
		document.getElementById("archive-current-config")?.addEventListener("click", async () => await this.archiveCurrentConfig());
		document.getElementById("refresh-configs")?.addEventListener("click", async () => await this.refreshConfigs());
		document.getElementById("open-configs-directory")?.addEventListener("click", async () => {
			const result = await invoke<CommandResult<null>>("open_directory", { dir: "configs" });
			if (result.error) logger.log(`Ошибка открытия директории конфигов: ${result.error}`, "error");
		});

		const configLoaded = await this.refreshConfigs();
		if (!configLoaded) await this.loadCurrentConfig();

		this.watch();
	}

	/** Метод запуска цикла сохранения пользовательского конфига */
	private watch = (): void => { setInterval(async () => await this.saveCurrentConfig(), 5000); };

	/** Метод сбора текущего конфига */
	private collectCurrentConfig(pub: boolean): string {
		const elements = document.querySelectorAll<HTMLElement>("[keep]");
		const config: Record<string, ConfigValue> = {};

		elements.forEach(e => {
			if (pub && e.getAttribute("private") !== null) return;

			const id = e.id.replaceAll("_", ".");

			if (e.tagName.toLocaleLowerCase() === "input") {
				const el = e as HTMLInputElement;
				el.type === "checkbox" ? config[id] = el.checked : config[id] = el.type === "number" ? el.value.includes(".") || el.value.includes(",") ? parseFloat(el.value) : parseInt(el.value) : el.value;
			} else if (e.tagName.toLocaleLowerCase() === "textarea") {
				const el = e as HTMLTextAreaElement;
				config[id] = el.type === "number" ? parseInt(el.value) : el.value;
			} else {
				const el = e as HTMLSelectElement;
				config[id] = el.selectedIndex;
			}
		});

		return JSON.stringify(config, null, 2);
	}

	/** Метод сохранения текущего пользовательского конфига */
	public async saveCurrentConfig(): Promise<boolean> {
		const config = this.collectCurrentConfig(false);

		const selectedConfigFilename = localStorage.getItem("selected-config-filename");
		if (selectedConfigFilename) {
			const result = await invoke<CommandResult<number[]>>("read_file", { path: `configs/${selectedConfigFilename}` });

			if (result.data && result.data.length > 0) {
				const dec = new TextDecoder("utf-8");
				const u8a = new Uint8Array(result.data);
				const dcd = dec.decode(u8a);

				if (JSON.stringify(config) !== JSON.stringify(dcd)) {
					localStorage.setItem("selected-config-changed", "true");
				} else {
					// Эта ветка по факту сильно ни на что не влияет, но по логике
					// здесь должно быть удалено поле "selected-config-changed" из
					// локального хранилища. Таким образом если пользователь изменит
					// импортированный конфиг и снова всё приведёт в исходный вид, будет
					// загружаться тот же импортированный конфиг, а не текущий. Даже
					// если это убрать - ничего не поменяется.
					localStorage.removeItem("selected-config-changed");
					return true;
				}
			}
		}

		const enc = new TextEncoder();
		const ecd = enc.encode(config);
		const arr = Array.from(ecd);

		const result = await invoke<CommandResult<null>>("s_save_file", {
			path: "current_config.json",
			data: arr,
		});

		if (result.error) logger.log(`Ошибка сохранения конфига: ${result.error}`, "error");

		return false;
	}

	/** Метод установки значения для элемента */
	private setValue(id: string, value: ConfigValue): void {
		if (id === "") return;

		try {
			const doc = document.getElementById(id.replaceAll(".", "_"));
			if (!doc) return;

			if (doc.tagName.toLocaleLowerCase() === "input") {
				const input = doc as HTMLInputElement;
				input.type === "checkbox" ? input.checked = Boolean(value) : typeof value === "number" ? input.valueAsNumber = value : input.value = value ? value.toString() : "";
			} else if (doc.tagName.toLocaleLowerCase() === "textarea") {
				const textarea = doc as HTMLTextAreaElement;
				textarea.value = value ? value.toString() : "";
			} else {
				const select = doc as HTMLSelectElement;
				if (typeof value === "number") select.selectedIndex = value;
			}
		} catch (error) {
			logger.log(`Ошибка установки значения для ${id}: ${error}`, "error");
		}
	}

	/** Метод обновления сохранённых конфигов */
	private async refreshConfigs(): Promise<boolean> {
		try {
			if (this.configList) this.configList.innerHTML = "";

			logger.log("Загрузка сохранённых конфигов...", "system");

			const result = await invoke<CommandResult<Record<string, number[]>>>("load_configs");

			if (result.error) {
				logger.log(`Ошибка загрузки сохранённых конфигов: ${result.error}`, "error");
				return false;
			}

			if (!result.data) return false;

			const selectedConfigFilename = localStorage.getItem("selected-config-filename");
			const selectedConfigChanged = localStorage.getItem("selected-config-changed") !== null;

			const decoder = new TextDecoder("utf-8");
			let loadedCount = 0;

			let currentConfig = null;
			let currentConfigFilename = null;

			for (const filename in result.data) {
				const data = result.data[filename];
				if (!data) continue;

				const u8a = new Uint8Array(data);
				const dcd = decoder.decode(u8a);
				const cfg = JSON.parse(dcd);
				if (!cfg) continue;

				if (selectedConfigFilename === filename && !selectedConfigChanged) {
					currentConfig = cfg;
					currentConfigFilename = filename;
				}

				this.createConfigCard(filename, selectedConfigFilename === filename);

				loadedCount++;
			}

			if (loadedCount > 0) {
				logger.log(`Сохранённые конфиги успешно загружены (${loadedCount} штук)`, "system");
				this.searchConfigs((document.getElementById("config-search-query") as HTMLInputElement).value);
			} else {
				logger.log("Сохранённые конфиги отсутствуют", "system");
			}

			if (currentConfig && currentConfigFilename) {
				for (const [id, value] of Object.entries<ConfigValue>(currentConfig)) this.setValue(id, value);
				triggerRegistry.triggerAll();
				logger.log(`Конфиг "${currentConfigFilename}" применён`, "system");
				return true;
			}
		} catch (err) {
			logger.log(`Ошибка загрузки сохранённых конфигов: ${err}`, "error");
		}

		return false;
	}

	/** Метод загрузки текущего конфига */
	private async loadCurrentConfig(): Promise<void> {
		try {
			const result = await invoke<CommandResult<number[]>>("s_read_file", { path: "current_config.json" });

			if (result.error) {
				logger.log(`Ошибка загрузки текущего конфига: ${result.error}`, "error");
				return;
			}

			if (!result.data || result.data.length < 1) return;

			logger.log("Загрузка текущего конфига...", "system");

			const dec = new TextDecoder("utf-8");
			const u8a = new Uint8Array(result.data);
			const dcd = dec.decode(u8a);
			const cfg = JSON.parse(dcd);

			if (cfg) {
				for (const [id, value] of Object.entries<ConfigValue>(cfg)) this.setValue(id, value);
				triggerRegistry.triggerAll();
				logger.log("Текущий конфиг успешно загружен и применён", "system");
			} else {
				logger.log("Не удалось загрузить текущий конфиг", "error");
			}
		} catch (err) {
			logger.log(`Ошибка загрузки текущего конфига: ${err}`, "error");
		}
	}

	/** Метод создания карточки конфига */
	private createConfigCard(filename: string, selected: boolean): void {
		if (!this.configList) return;

		const configName = filename.slice(0, filename.length - 5);

		this.configs.set(configName, selected);

		const configCard = document.createElement("div");
		configCard.innerHTML = configName;

		selected ? configCard.classList.add("selected") : configCard.classList.remove("selected");

		configCard.addEventListener("click", async () => {
			if (localStorage.getItem("selected-config-filename") === filename) return;

			const result = await invoke<CommandResult<number[]>>("s_read_file", { path: `configs/${filename}` });

			if (result.error) {
				logger.log(`Ошибка применения конфига "${filename}": ${result.error}`, "error");
				return;
			}

			if (!result.data || result.data.length < 1) return;

			for (const c of document.querySelectorAll<HTMLDivElement>("#config-list div")) {
				if (c.innerText === configName) continue;
				c.classList.remove("selected");
				this.configs.set(c.innerText, false);
			}

			const decoder = new TextDecoder("utf-8");
			const uint8arr = new Uint8Array(result.data);
			const decoded = decoder.decode(uint8arr);
			const config = JSON.parse(decoded);

			for (const [id, value] of Object.entries<ConfigValue>(config)) this.setValue(id, value);
			triggerRegistry.triggerAll();

			const current = this.configs.get(configName);
			current ? configCard.classList.remove("selected") : configCard.classList.add("selected");
			this.configs.set(configName, !current);

			localStorage.setItem("selected-config-filename", filename);
			localStorage.removeItem("selected-config-changed");

			logger.log(`Конфиг "${filename}" успешно применён`, "system");
		});

		this.configList.appendChild(configCard);
	}

	/** Метод импорта сторонних конфигов */
	private async importConfigs(): Promise<void> {
		try {
			const paths = await open({
				directory: false,
				multiple: true,
				filters: [{
					name: "Configs",
					extensions: ["json"]
				}]
			});

			if (!paths || paths.length < 1) return;

			let imported = 0;

			for (const path of paths) {
				const filename = path.split(/[\\/]/).pop();
				if (!filename) continue;

				const buf = await invoke<number[]>("read_file", { path: path });
				if (buf.length < 1) return;

				const result = await invoke<CommandResult<null>>("s_save_file", {
					path: `configs/${filename}`,
					data: Uint8Array.from(buf),
				});

				if (result.error) {
					logger.log(`Ошибка сохранения конфига "${filename}": ${result.error}`, "error");
				} else {
					imported++;
				}
			}

			await this.refreshConfigs();

			messages.message("Cấu hình", `Đã nhập ${imported} cấu hình`);
		} catch (err) {
			logger.log(`Не удалось импортировать указанный конфиг(и): ${err}`, "error");
		}
	}

	/** Метод экспорта текущего пользовательского конфига */
	private async exportCurrentConfig(): Promise<void> {
		try {
			const directory = await open({
				directory: true,
				multiple: false
			});

			if (!directory) return;

			const cfg = this.collectCurrentConfig(true);
			const enc = new TextEncoder();
			const ecd = enc.encode(cfg);
			const arr = Array.from(ecd);

			const result = await invoke<CommandResult<string>>("export_config", {
				directory: directory,
				content: arr,
			});

			if (result.error) {
				logger.log(`Ошибка экспорта текущего конфига в ${directory}: ${result.error}`, "error");
				messages.message("Cấu hình", `Lỗi xuất cấu hình vào ${directory}`);
				return;
			}

			if (result.data) {
				logger.log(`Публичный конфиг "${result.data}" успешно сохранён в директорию ${directory}`, "system");
				messages.message("Cấu hình", `Cấu hình công khai đã được lưu vào file ${result.data}`);
			}
		} catch (err) {
			logger.log(`Ошибка экспорта публичного конфига: ${err}`, "error");
		}
	}

	/** Метод архивации текущего пользовательского конфига */
	public async archiveCurrentConfig(): Promise<void> {
		try {
			const cfg = this.collectCurrentConfig(false);
			const enc = new TextEncoder();
			const ecd = enc.encode(cfg);
			const arr = Array.from(ecd);

			const result = await invoke<CommandResult<string>>("archive_config", { content: arr });

			if (result.error) {
				logger.log(`Ошибка архивации текущего конфига: ${result.error}`, "error");
				return;
			}

			if (result.data) {
				logger.log(`Текущий конфиг успешно архивирован в файл "${result.data}"`, "system");
				await this.refreshConfigs();
			}
		} catch (err) {
			logger.log(`Ошибка архивации текущего конфига: ${err}`, "error");
		}
	}

	/** Метод поиска конфига */
	private searchConfigs(query: string): void {
		const cards = document.querySelectorAll<HTMLDivElement>("#config-list div");

		if (query === "") {
			cards.forEach(c => c.style.display = "flex");
		} else {
			cards.forEach(c => c.style.display = c.innerText.toLowerCase().includes(query.toLowerCase()) ? "flex" : "none");
		}
	}
}

const configModule = new ConfigModule();

export { configModule }
