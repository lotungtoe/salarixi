import { readFile } from "@tauri-apps/plugin-fs";

import { logger } from "../utils/logger";
import { messages } from "../utils/message";
import { open } from "@tauri-apps/plugin-dialog";
import { triggerRegistry } from "../trigger/registry";
import { invoke } from "@tauri-apps/api/core";

/** Тип значения поля конфигурации */
type ConfigValue = string | number | boolean | null;

/** Модуль управления пользовательской конфигурацией */
class ConfigModule {
  /** Метод инициализации конфигуратора */
  public async init(): Promise<void> {
    document.getElementById("upload-config")?.addEventListener("click", async () => await this.uploadConfig());
    document.getElementById("share-config")?.addEventListener("click", async () => await this.shareConfig());

    setInterval(async () => {
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
      const uint8 = encoder.encode(JSON.stringify(config));

      const result = await invoke("save_config", {
        bytes: Array.from(uint8),
      }) as CommandResult<null>;

      if (result.error) {
        logger.log(`Ошибка сохранения конфига: ${result.error}`, "error");
      }
    }, 1500);

    logger.log("Загрузка конфига...", "system");

    const result = await invoke("load_config") as CommandResult<number[]>;

    if (!result.data) {
      logger.log(`Ошибка загрузки конфига: ${result.error}`, "error");
      return;
    }

    if (result.data.length < 1) return;

    const uint8 = new Uint8Array(result.data);
    const decoder = new TextDecoder("utf-8");
    const config = decoder.decode(uint8);

    for (const [id, value] of Object.entries<ConfigValue>(JSON.parse(config))) this.setValue(id, value);

    triggerRegistry.triggerAll();

    logger.log("Конфиг успешно загружен", "system");
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

  /** Метод загрузки стороннего конфига */
  private async uploadConfig(): Promise<void> {
    try {
      const path = await open({
        directory: false,
        multiple: false,
        filters: [{
          name: "Config",
          extensions: ["json"]
        }]
      });

      if (!path) return;

      const buffer = await readFile(path);
      if (!buffer) return;

      const decoder = new TextDecoder();
      const config = JSON.parse(decoder.decode(buffer));

      for (const [id, value] of Object.entries<ConfigValue>(config)) value || value === 0 ? this.setValue(id, value) : null;

      triggerRegistry.triggerAll();

      messages.message("Конфиг", `Конфиг успешно загружен из ${path}`);
    } catch (error) {
      logger.log(`Не удалось загрузить конфиг: ${error}`, "error");
    }
  }
  
  /** Метод создания публичного конфига */
  private async shareConfig(): Promise<void> {
    try {
      const directory = await open({
        directory: true,
        multiple: false
      });

      if (!directory) return;

      const result = await invoke("create_public_config", {
        directory: directory,
      }) as CommandResult<null>;

      if (result.error) {
        logger.log(`Ошибка создания публичного конфига в ${directory}: ${result.error}`, "error");
        messages.message("Конфиг", `Ошибка создания публичного конфига в ${directory}`);
      }

      messages.message("Конфиг", `Публичный конфиг успешно создан в ${directory}`);
    } catch (error) {
      logger.log(`Не удалось поделиться конфигом: ${error}`, "error");
    }
  }
}

const configModule = new ConfigModule();

export { configModule }
