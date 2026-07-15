import { readFile } from "@tauri-apps/plugin-fs";

import { logger } from "../utils/logger";
import { messages } from "../utils/message";
import { open } from "@tauri-apps/plugin-dialog";
import { triggerRegistry } from "../trigger/registry";
import { invoke } from "@tauri-apps/api/core";

/** Тип значения поля конфигурации */
type ConfigValue = string | number | boolean | null;

/** Метаданные конфига */
interface ConfigMeta {
  selected: boolean;
}

/** Модуль управления пользовательской конфигурацией */
class ConfigModule {
  private configList: HTMLDivElement | null = null;
  private configs: Map<string, ConfigMeta> = new Map;

  /** Метод инициализации конфигуратора */
  public async init(): Promise<void> {
    this.configList = document.getElementById("config-list") as HTMLDivElement | null;

    const initializeResult = await invoke("initialize_configs_dir") as CommandResult<null>;
    if (initializeResult.error) {
      logger.log(`Ошибка инициализации конфигуратора: ${initializeResult.error}`, "error");
      return;
    }

    const configSearchQuery = document.getElementById("config-search-query") as HTMLInputElement;
    document.getElementById("search-configs")?.addEventListener("click", () => this.searchConfigs(configSearchQuery.value));

    document.getElementById("import-configs")?.addEventListener("click", async () => await this.importConfigs());
    document.getElementById("export-current-config")?.addEventListener("click", async () => await this.exportCurrentConfig());
    document.getElementById("refresh-configs")?.addEventListener("click", async () => await this.refreshConfigs());
    document.getElementById("open-configs-directory")?.addEventListener("click", async () => {
      const result = await invoke("open_directory", { dir: "configs" }) as CommandResult<null>;
      if (result.error) logger.log(`Ошибка открытия директории конфигов: ${result.error}`, "error");
    });

    const configLoaded = await this.refreshConfigs();
    if (!configLoaded) await this.loadCurrentConfig();

    setInterval(async () => await this.saveCurrentConfig(), 5000);
  }

  /** Метод сохранения текущего конфига */
  public async saveCurrentConfig(): Promise<void> {
    const elements = document.querySelectorAll<HTMLElement>('[keep="true"]');
    const config: Record<string, ConfigValue> = {};

    elements.forEach(e => {
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

    const encoder = new TextEncoder();
    const encoded = encoder.encode(JSON.stringify(config));
    const content = Array.from(encoded);

    const result = await invoke("save_file", {
      path: "current_config.json",
      content: content,
    }) as CommandResult<null>;

    if (result.error) logger.log(`Ошибка сохранения конфига: ${result.error}`, "error");
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

      const result = await invoke("load_configs") as CommandResult<Record<string, number[]>>;

      if (result.error) {
        logger.log(`Ошибка загрузки сохранённых конфигов: ${result.error}`, "error");
        return false;
      }

      if (!result.data) return false;

      const selectedConfigFilename = localStorage.getItem("selected-config-filename");

      const decoder = new TextDecoder("utf-8");
      let loadedCount = 0;

      let currentConfig = null;
      let currentConfigFilename = null;

      for (const filename in result.data) {
        const data = result.data[filename];
        if (!data) continue;

        const uint8arr = new Uint8Array(data);
        const decoded = decoder.decode(uint8arr);
        const config = JSON.parse(decoded);
        if (!config) continue;

        if (selectedConfigFilename === filename) {
          currentConfig = config;
          currentConfigFilename = filename;
        }

        this.createConfigCard(filename, selectedConfigFilename === filename);

        loadedCount++;
      }

      if (loadedCount > 0) logger.log(`Сохранённые конфиги успешно загружены (${loadedCount} штук)`, "system");

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
      const result = await invoke("read_file", { path: "current_config.json" }) as CommandResult<number[]>;
    
      if (result.error) {
        logger.log(`Ошибка загрузки текущего конфига: ${result.error}`, "error");
        return;
      }

      if (!result.data || result.data.length < 1) return;

      logger.log("Загрузка текущего конфига...", "system");

      const decoder = new TextDecoder("utf-8");
      const uint8arr = new Uint8Array(result.data);
      const decoded = decoder.decode(uint8arr);
      const parsed = JSON.parse(decoded);

      if (parsed) {
        for (const [id, value] of Object.entries<ConfigValue>(parsed)) this.setValue(id, value);
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

    this.configs.set(filename, {
      selected: selected,
    });

    const card = document.createElement("div");
    card.className = "card";
    card.innerHTML = `
      <label class="filename">${filename}</label>
      <label class="tag">${selected ? "SELECTED" : ""}</label>
    `;

    card.addEventListener("click", async () => {
      if (localStorage.getItem("selected-config-filename") === filename) return;
  
      const tag = card.querySelector<HTMLLabelElement>(".tag");
      if (!tag) return;

      const meta = this.configs.get(filename);
      if (!meta) return;

      const result = await invoke("read_file", { path: `configs/${filename}` }) as CommandResult<number[]>;

      if (result.error) {
        logger.log(`Ошибка применения конфига "${filename}": ${result.error}`, "error");
        return;
      }

      if (!result.data || result.data.length < 1) return;

      for (const header of document.querySelectorAll<HTMLDivElement>("#config-list .card")) {
        const f = header.querySelector<HTMLLabelElement>(".filename");
        if (!f || f.innerText === filename) continue;

        const t = header.querySelector<HTMLLabelElement>(".tag");
        if (!t) continue;
        t.innerText = "";

        const meta = this.configs.get(f.innerText);
        if (!meta) continue;
        meta.selected = false;
      }

      const decoder = new TextDecoder("utf-8");
      const uint8arr = new Uint8Array(result.data);
      const decoded = decoder.decode(uint8arr);
      const parsed = JSON.parse(decoded);
      const config = parsed["options"] ?? parsed["settings"] ?? parsed["config"] ?? parsed["cfg"] ?? parsed["content"] ?? parsed;

      for (const [id, value] of Object.entries<ConfigValue>(config)) this.setValue(id, value);
      triggerRegistry.triggerAll();

      meta.selected = !meta.selected;
      tag.innerText = meta.selected ? "SELECTED" : "";

      logger.log(`Конфиг "${filename}" успешно применён`, "system");
      localStorage.setItem("selected-config-filename", filename);
    });

    this.configList.appendChild(card);
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

        const buf = await readFile(path);
        if (!buf) return;

        const result = await invoke("save_file", {
          path: `configs/${filename}`,
          content: buf,
        }) as CommandResult<null>;

        if (result.error) {
          logger.log(`Ошибка сохранения конфига "${filename}": ${result.error}`, "error");
        } else {
          this.createConfigCard(filename, false);
          imported++;
        }
      }

      messages.message("Конфиг", `Импортировано ${imported} конфигов`);
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

      const result = await invoke("create_public_config", { directory: directory }) as CommandResult<string>;

      if (result.error) {
        logger.log(`Ошибка создания публичного конфига в ${directory}: ${result.error}`, "error");
        messages.message("Конфиг", `Ошибка создания публичного конфига в ${directory}`);
        return;
      }

      messages.message("Конфиг", `Публичный конфиг успешно сохранён в файл ${result.data}`);
    } catch (err) {
      logger.log(`Не удалось поделиться конфигом: ${err}`, "error");
    }
  }

  /** Метод поиска конфига */
  private searchConfigs(query: string): void {
    const cards = document.querySelectorAll<HTMLDivElement>("#config-list .card");
    
    if (query === "") {
      cards.forEach(card => card.style.display = "flex");
    } else {
      cards.forEach(card => {
        const filename = card.querySelector<HTMLLabelElement>(".filename");
        if (!filename) return;
        card.style.display = filename.innerText.includes(query) ? "flex" : "none";
      });
    }
  }
}

const configModule = new ConfigModule();

export { configModule }
