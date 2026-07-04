import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { addOpeningUrlTo } from "../main";
import { getCacheOrNew, LcfResult, LnfResult, loadCache, saveCache, TimeType } from "../utils/cache";

/** Структура информации клиента */
interface ClientInfo {
  downloads: number;
  releases: number;
  social: any;
}

/** Небольшой модуль управления информацией клиента */
class InfoModule {
  /** Метод инициализации модуля */
  public async init(): Promise<void> {
    try {
      logger.log("Загрузка информации о клиенте...", "system");

      // Думаю нету смысла хранить актуальную ссылку на репозиторий, так как 
      // если он будет перенесён - загрузить актуальную ссылку не получится
      addOpeningUrlTo("codeberg", "click", "https://codeberg.org/nullclyze/salarixi");

      const releaseCountEl = document.getElementById("release-count") as HTMLSpanElement;
      const startCountEl = document.getElementById("stars-count") as HTMLSpanElement;
      const forksCountEl = document.getElementById("forks-count") as HTMLSpanElement;

      const onerr = (err: string) => {
        ["telegram", "discord", "youtube"].forEach(id => document.getElementById(id)?.remove());
        logger.log(`Ошибка загрузки информации о клиенте (не критично): ${err}`, "non-critical-error");
      };

      const result = await getCacheOrNew<ClientInfo>(this.lcf, this.lnf, this.scf, onerr);
      if (!result.data) return;

      releaseCountEl.innerText = result.data.repo.releases > 0 ? String(result.data.repo.releases) : "?";
      startCountEl.innerText = result.data.repo.stars > 0 ? String(result.data.repo.stars) : "?";
      forksCountEl.innerText = result.data.repo.forks > 0 ? String(result.data.repo.forks) : "?";

      for (const key in result.data.social) {
        const link = result.data.social[key] as string;
        addOpeningUrlTo(key, "click", link);
      }

      logger.log(`Информация о клиенте загружена (размер ${result.size})`, "system");
    } catch (error) {
      logger.log(`Ошибка инициализации информации о клиенте (не критично): ${error}`, "non-critical-error");
    }
  }

  /** Вспомогательный метод конвертации числа скачиваний в строку */
  private downloadCountToString(count: number): string {
    if (count < 1000) {
      return count.toString();
    } else if (count >= 1000 && count < 1_000_000) {
      return (count / 1000).toFixed(2) + "k";
    } else {
      return (count / 1_000_000).toFixed(2) + "M";
    }
  }

  /** Метод загрузки актуальной информации о клиенте */
  private async lnf(): Promise<LnfResult> {
    try {
      const repository = await invoke("download_json", {
        url: "https://codeberg.org/api/v1/repos/nullclyze/salarixi"
      }) as any;

      if (!repository) return { data: null, error: "resource moved or unavailable" };
    
      const social = await invoke("download_json", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.social.json"
      }) as any;

      if (!social) return { data: null, error: "resource moved or unavailable" };

      return {
        data: {
          repo: {
            releases: repository["release_counter"] ?? repository["release_count"] ?? repository["releases"] ?? 0,
            stars: repository["stars_count"] ?? repository["stars"] ?? 0,
            forks: repository["forks_count"] ?? repository["forks"] ?? 0,
          },
          social: social,
        },
        error: null,
      };
    } catch (err) {
      return { data: null, error: String(err) };
    }
  }

  /** Метод загрузки кэшированной информации о клиенте */
  private async lcf(): Promise<LcfResult> {
    const result = await loadCache("client-info", "client-info.json", 3, TimeType.Day);
    if (!result.data) return { data: null, expired: false };

    return {
      data: JSON.parse(result.data),
      expired: result.error === "expired",
    };
  }

  /** Метод кэширования информации */
  private async scf(info: ClientInfo): Promise<void> {
    const result = await saveCache(JSON.stringify(info, null, 2), "client-info.json");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования информации о клиенте (не критично): ${result.error}`, "non-critical-error");
    }
  }
}

const infoModule = new InfoModule();

export { infoModule }
