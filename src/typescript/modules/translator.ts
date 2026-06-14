import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";
import { loadCache, saveCache, TimeType } from "../utils/cache";
import { getStringSize } from "../utils/size";

/** Код языка */
export type Language = "ru" | "en" | "tr";

/** Модуль переподчика */
class TranslatorModule {
  /** Метод инициализации переводчика */
  public async init(): Promise<void> {
    try {
      logger.log("Загрузка перевода...", "system");

      const langSelect = document.getElementById("interface_select_client-language") as HTMLSelectElement;
      const cachedTranslation = await this.loadCachedTranslation();
      let dataSize = 0;

      if (cachedTranslation) {
        dataSize = getStringSize(JSON.stringify(cachedTranslation, null, 2));
        langSelect.addEventListener("change", () => this.translate(langSelect.value as Language, cachedTranslation));
        if (langSelect.value !== "ru") this.translate(langSelect.value as Language, cachedTranslation);
      } else {
        const newTranslation = await this.loadNewTranslation();

        if (newTranslation) {
          dataSize = getStringSize(JSON.stringify(newTranslation, null, 2));
          langSelect.addEventListener("change", () => this.translate(langSelect.value as Language, newTranslation));
          if (langSelect.value !== "ru") this.translate(langSelect.value as Language, newTranslation);
          await this.cacheTranslation(newTranslation);
        } else {
          return;
        }
      }

      const prettyDataSize = (dataSize / 1024).toFixed(1);

      logger.log(`Перевод загружен (размер ${prettyDataSize}KB)`, "system");
    } catch (error) {
      logger.log(`Ошибка загрузки перевода: ${error}`, "error");
    }
  }

  /** Метод перевода текста на другой язык */
  private translate(lang: Language, translation: any): void {
    try {
      const elements = translation["map"] ?? translation["array"] ?? translation["list"] ?? translation;
      document.querySelectorAll<HTMLElement>("[translator-tag]").forEach(e => {
        let tag = e.getAttribute("translator-tag");
        if (!tag) return;
        for (let el of elements) el.tags.includes(tag) && el.lang[lang] ? e.innerText = el.lang[lang] : null;
      });
    } catch (error) {
      logger.log(`Ошибка перевода: ${error}`, "error");
    }
  }

  /** Метод загрузки актуального перевода */
  private async loadNewTranslation(): Promise<any> {
    try {
      const translation = await invoke("download_json", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.lang.json"
      }) as any;

      if (!translation) {
        logger.log("Ошибка загрузки перевода (не критично): Resource moved or is unavailable", "error");
        return null;
      }

      return translation;
    } catch (error) {
      logger.log(`Ошибка загрузки перевода (не критично): ${error}`, "error");
    }

    return null;
  }

  /** Метод загрузки кэшированного перевода */
  public async loadCachedTranslation(): Promise<any> {
    const result = await loadCache("translation", "translation.json", 3, TimeType.Day);
    if (!result.data) return null;
    return JSON.parse(result.data);
  }

  /** Метод кэширования перевода */
  private async cacheTranslation(translation: any): Promise<void> {
    const result = await saveCache(JSON.stringify(translation, null, 2), "translation.json");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования перевода (не критично): ${result.error}`, "error");
      return;
    }
  }
}

const translatorModule = new TranslatorModule();

export { translatorModule }
