import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";
import { loadCache, saveCache, TimeType } from "../utils/cache";
import { getStringSize } from "../utils/size";
import { currentClientVersion } from "../main";
import { isVersionNewer } from "../utils/semver";

/** Модуль переподчика */
class TranslatorModule {
  /** Метод инициализации переводчика */
  public async init(): Promise<void> {
    try {
      logger.log("Загрузка перевода...", "system");

      const langSelect = document.getElementById("interface_select_language") as HTMLSelectElement;
      const cachedTranslation = await this.loadCachedTranslation();
      let data = null;

      if (cachedTranslation) {
        data = cachedTranslation;
      } else {
        const newTranslation = await this.loadNewTranslation();

        if (newTranslation) {
          data = newTranslation;
          await this.cacheTranslation(newTranslation);
        } else {
          return;
        }
      }

      const langs = data["languages"] ?? data["langs"] ?? data["translations"];

      if (!langs) {
        logger.log("Ошибка загрузки перевода: Information is corrupted or missing", "error");
        return;
      }

      const translation = data["data"] ?? data["translation"] ?? data["list"] ?? data["map"] ?? data["array"];

      if (!translation) {
        logger.log("Ошибка загрузки перевода: Information is corrupted or missing", "error");
        return;
      }

      for (const lang of langs) {
        const code = lang["code"] ?? lang["tag"];
        const name = lang["name"] ?? lang["label"] ?? lang["kind"] ?? code;

        if (!code || !name) continue;

        const codeStr = String(code);
        const nameStr = String(name);

        langSelect.innerHTML += `<option value="${codeStr}">${codeStr.toUpperCase()} (${nameStr})</option>`;
      } 

      const selectedLang = localStorage.getItem("language");
      if (selectedLang) langSelect.selectedIndex = parseInt(selectedLang);

      langSelect.addEventListener("change", () => this.translate(langSelect.value, langSelect.selectedIndex, translation));
      if (langSelect.value !== "ru") this.translate(langSelect.value, langSelect.selectedIndex, translation);

      const dataSize = getStringSize(JSON.stringify(data, null, 2));
      const prettyDataSize = (dataSize / 1024).toFixed(1);

      logger.log(`Перевод загружен (размер ${prettyDataSize}KB)`, "system");
    } catch (error) {
      logger.log(`Ошибка загрузки перевода: ${error}`, "error");
    }
  }

  /** Метод перевода текста на другой язык */
  private translate(targetLanguage: string, selectedIndex: number, translation: any): void {
    try {
      let filteredTranslation = [];

      // Для начала нужно пройтись по всем элементам перевода и отфильтровать
      // их, чтобы после не бегать по всем (нужным и ненужным), это ускорит процесс
      for (const piece of translation) {
        const versions = piece["versions"] ?? piece["scope"] ?? piece["rules"];
        if (!versions) continue;

        // Могут быть такие комибнации:
        // 1. 1.0.0 (только 1.0.0)
        // 2. 1.0.0:1.8.5 (с 1.0.0 по 1.8.5 включительно)
        // 3. 1.0.0:1.8.5+1.9.1:2.4.3 (с 1.0.0 по 1.8.5 включительно и с 1.9.1 по 2.4.3 включительно)
        // 3. 1.0.0:1.8.5+1.9.1 (с 1.0.0 по 1.8.5 включительно и 1.9.1)

        const fragments = String(versions).split("+");

        for (const fragment of fragments) {
          const split = fragment.split(":");
          const minimum = split[0];
          const maximum = split[1];

          if (!maximum && minimum === currentClientVersion) {
            filteredTranslation.push(piece);
            break;
          } else if ((minimum === currentClientVersion || !isVersionNewer(minimum)) && (maximum === currentClientVersion || isVersionNewer(maximum))) {
            filteredTranslation.push(piece);
            break;
          }
        }
      }

      document.querySelectorAll<HTMLElement>("[tt]").forEach(e => {
        let [tag, isColon] = [e.getAttribute("tt"), false];
        if (!tag) return;
        if (tag[tag.length - 1] === ":") [tag, isColon] = [tag.slice(0, tag.length - 1), true];

        for (const piece of filteredTranslation) {
          const pieceTag = piece["tag"] ?? piece["id"] ?? piece["name"] ?? piece["mark"];
          const pieceLang = piece["lang"] ?? piece["langs"] ?? piece["options"];
          if (pieceTag !== tag || !pieceLang[targetLanguage]) continue;

          if (isColon) {
            e.innerText = `${pieceLang[targetLanguage]}:`;
          } else {
            e.innerText = pieceLang[targetLanguage];
          }

          break;
        }
      });

      localStorage.setItem("language", selectedIndex.toString());
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
        logger.log("Ошибка загрузки перевода: Resource moved or is unavailable", "error");
        return null;
      }

      return translation;
    } catch (error) {
      logger.log(`Ошибка загрузки перевода: ${error}`, "error");
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
      logger.log(`Ошибка кэширования перевода: ${result.error}`, "error");
      return;
    }
  }
}

const translatorModule = new TranslatorModule();

export { translatorModule }
