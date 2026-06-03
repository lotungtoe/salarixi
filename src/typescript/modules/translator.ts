import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";

export type Language = "ru" | "en" | "tr";

class Translator {
  private cache: any;

  constructor() {
    this.cache = null;
  }

  /** Метод инициализации функций, связанных с переводчиком. */
  public async init(): Promise<void> {
    let langSelect = document.getElementById("interface_select_client-language") as HTMLSelectElement;
    langSelect.addEventListener("change", async () => await this.translate(langSelect.value as Language));
    if (langSelect.value !== "ru") await this.translate(langSelect.value as Language);
  }

  /** Метод получения кэша. */
  public getCache(): any {
    return this.cache;
  }

  /** Метод получения и установки актуального перевода. */
  private async translate(lang: Language): Promise<void> {
    try {
      if (!this.cache) {
        let content = await invoke("download_json", {
          url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.lang.json"
        }) as any;

        if (content) {
          this.cache = content["map"];
        } else {
          logger.log(`Ошибка загрузки перевода: Failed to load JSON-content`, "error");
          return;
        }
      }

      document.querySelectorAll<HTMLElement>("[translator-tag]").forEach(e => {
        let tag = e.getAttribute("translator-tag");
        if (!tag) return;
        for (let el of this.cache) el.tags.includes(tag) && el.lang[lang] ? e.innerText = el.lang[lang] : null;
      });
    } catch (error) {
      logger.log(`Ошибка перевода: ${error}`, "error");
    }
  }
}

const translator = new Translator();

export { translator }