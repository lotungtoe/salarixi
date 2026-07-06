import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";

import { currentClientVersion } from "../main";
import { isVersionNewer } from "../utils/semver";
import { loadCacheOrNew, LcfResult, LnfResult, loadCache, saveCache, TimeType } from "../utils/cache";

/** Модуль переподчика */
class TranslatorModule {
  /** Метод инициализации переводчика */
  public async init(): Promise<void> {
    try {
      logger.log("Загрузка перевода...", "system");
      
      const langSelect = document.getElementById("interface_select_language") as HTMLSelectElement;

      const onerr = (err: string) => logger.log(`Ошибка загрузки перевода: ${err}`, "error");
      const result = await loadCacheOrNew<any>(this.lcf, this.lnf, this.scf, onerr);
      if (!result.data) return;

      const data = result.data;
      const langs = data["languages"] ?? data["langs"] ?? data["translations"];
      const translation = data["data"] ?? data["translation"] ?? data["list"] ?? data["map"] ?? data["array"];

      if (!langs || !translation) {
        logger.log("Ошибка загрузки перевода: information is corrupted or missing", "error");
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

      langSelect.addEventListener("change", async () => await this.translate(langSelect.value, langSelect.selectedIndex, translation));
      if (langSelect.value !== "ru") await this.translate(langSelect.value, langSelect.selectedIndex, translation);

      logger.log(`Перевод загружен (размер ${result.size})`, "system");
    } catch (error) {
      logger.log(`Ошибка загрузки перевода: ${error}`, "error");
    }
  }

  /** Метод перевода текста на другой язык */
  private async translate(targetLanguage: string, selectedIndex: number, translation: any): Promise<void> {
    try {
      const isDevMode = await invoke("is_dev_mode") as boolean;
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
        // 4. 1.0.0:1.8.5+1.9.1 (с 1.0.0 по 1.8.5 включительно и 1.9.1)
        // 5. 0.0.0 (текущая dev-версия без явного идентификатора)
        // 6. 1.0.0:0.0.0 (с 1.0.0 по текущую dev-версию включительно)

        const fragments = String(versions).split("+");

        for (const fragment of fragments) {
          const split = fragment.split(":");
          const minimum = split[0];
          const maximum = split[1];

          if (isDevMode && ((minimum === "0.0.0" || minimum === "dev") || (maximum === "0.0.0" || maximum === "dev"))) {
            filteredTranslation.push(piece);
            break;
          }

          if (!maximum && minimum && minimum === currentClientVersion) {
            filteredTranslation.push(piece);
            break;
          }

          if (
            minimum && (minimum === currentClientVersion || !isVersionNewer(minimum))
            &&
            maximum && (maximum === currentClientVersion || isVersionNewer(maximum))
          ) {
            filteredTranslation.push(piece);
            break;
          }
        }
      }

      document.querySelectorAll<HTMLElement>("[tt]").forEach(e => {
        let [tag, isColon] = [e.getAttribute("tt"), false];
        if (!tag) return;

        try {
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
        } catch (err) {
          logger.log(`Ошибка перевода элемента "${tag}": ${err}`, "error");
        }
      });

      localStorage.setItem("language", selectedIndex.toString());
    } catch (error) {
      logger.log(`Ошибка перевода: ${error}`, "error");
    }
  }

  /** Метод загрузки актуального перевода */
  private async lnf(): Promise<LnfResult> {
    try {
      const translation = await invoke("download_json", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.lang.json"
      }) as any;

      return { data: translation, error: translation ? null : "resource moved or unavailable" };
    } catch (err) {
      return { data: null, error: String(err) };
    }
  }

  /** Метод загрузки кэшированного перевода */
  public async lcf(): Promise<LcfResult> {
    const result = await loadCache("translation", "translation.json", 3, TimeType.Day);
    if (!result.data) return { data: null, incompatible: false, expired: false };

    return {
      data: JSON.parse(result.data),
      incompatible: result.error === "incompatible",
      expired: result.error === "expired",
    };
  }

  /** Метод кэширования перевода */
  private async scf(translation: any): Promise<void> {
    const result = await saveCache(JSON.stringify(translation, null, 2), "translation.json");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования перевода: ${result.error}`, "error");
    }
  }
}

const translatorModule = new TranslatorModule();

export { translatorModule }
