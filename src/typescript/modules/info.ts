import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { loadCache, saveCache, TimeType } from "../utils/cache";
import { addOpeningUrlTo } from "../main";

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
      // Думаю нету смысла хранить актуальную ссылку на репозиторий, так как 
      // если он будет перенесён - загрузить актуальную ссылку не получится
      addOpeningUrlTo("codeberg", "click", "https://codeberg.org/nullclyze/salarixi");

      const downloadCountEl = document.getElementById("download-count") as HTMLSpanElement;
      const releaseCountEl = document.getElementById("release-count") as HTMLSpanElement;

      const cachedInfo = await this.loadCachedClientInfo();
      let clientInfo = null;

      if (cachedInfo) {
        clientInfo = cachedInfo;
      } else {
        const newInfo = await this.loadNewClientInfo();

        if (newInfo) {
          clientInfo = newInfo;
          await this.cacheClientInfo(newInfo);
        }
      }

      if (!clientInfo) {
        ["telegram", "discord", "youtube"].forEach(id => document.getElementById(id)?.remove());
        logger.log("Ошибка инициализации информации о клиенте (не критично): Information is corrupted or missing", "error");
        return;
      }

      downloadCountEl.innerHTML = this.downloadCountToString(clientInfo.downloads);
      releaseCountEl.innerHTML = clientInfo.releases.toString();

      for (const key in clientInfo.social) {
        const link = clientInfo.social[key] as string;
        addOpeningUrlTo(key, "click", link);
      }
    } catch (error) {
      logger.log(`Ошибка инициализации информации о клиенте (не критично): ${error}`, "error");
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
  private async loadNewClientInfo(): Promise<ClientInfo | null> {
    try {
      const releases = await invoke("download_json", {
        url: "https://codeberg.org/api/v1/repos/nullclyze/salarixi/releases"
      }) as any;

      if (!releases) {
        logger.log("Ошибка загрузки информации о клиенте (не критично): Resource moved or is unavailable", "error");
        return null;
      }
    
      const social = await invoke("download_json", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.social.json"
      }) as any;

      if (!social) {
        logger.log("Ошибка загрузки информации о клиенте (не критично): Resource moved or is unavailable", "error");
        return null;
      }

      let totalDownloadCount = 0;
      let totalReleaseCount = 0;

      for (const release of releases) {
        totalReleaseCount++;
        for (const build of release["assets"]) totalDownloadCount += build["download_count"] ? build["download_count"] as number : 0;
      }

      return {
        downloads: totalDownloadCount,
        releases: totalReleaseCount,
        social: social,
      };
    } catch (error) {
      logger.log(`Ошибка загрузки информации о клиенте (не критично): ${error}`, "error");
    }

    return null;
  }

  /** Метод загрузки кэшированной информации о клиенте */
  private async loadCachedClientInfo(): Promise<ClientInfo | null> {
    const result = await loadCache("client-info", "client-info.json", 2, TimeType.Day);
    if (!result.data) return null;
    return JSON.parse(result.data);
  }

  /** Метод кэширования информации */
  private async cacheClientInfo(info: ClientInfo): Promise<void> {
    const result = await saveCache(JSON.stringify(info, null, 2), "client-info.json");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования информации о клиенте (не критично): ${result.error}`, "error");
      return;
    }
  }
}

const infoModule = new InfoModule();

export { infoModule }
