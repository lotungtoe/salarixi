import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { loadCache, saveCache, TimeType } from "../utils/cache";
import { addOpeningUrlTo, currentClientVersion } from "../main";

/** Небольшой модуль управления обновлениями клиента */
class UpdaterModule {
  /** Метод инициализации модуля */
  public init(): void {
    setTimeout(async () => {
      try {
        const notice = document.getElementById("notice") as HTMLElement;
        const newVersion = document.getElementById("new-client-version") as HTMLElement;
        const newTag = document.getElementById("new-client-tag") as HTMLElement;
        const newReleaseDate = document.getElementById("new-client-release-date") as HTMLElement;

        document.getElementById("close-notice-btn")?.addEventListener("click", () => notice.remove());

        const cachedVersion = await this.loadCachedVersionInfo();
        let latestVersion = null;

        if (cachedVersion) {
          latestVersion = cachedVersion;
        } else {
          const newVersion = await this.loadNewVersionInfo();

          if (newVersion) {
            latestVersion = newVersion;
            await this.cacheVersionInfo(newVersion);
          }
        }

        if (!latestVersion) {
          notice.remove();
          logger.log("Ошибка проверки обновлений (не критично): Information is corrupted or missing", "error");
          return;
        }

        if (latestVersion["version"] !== currentClientVersion) {
          const tag = `v${latestVersion.version}-${String(latestVersion.type).toLowerCase()}`;

          newVersion.innerText = latestVersion["version"];
          newTag.innerText = tag;
          newReleaseDate.innerText = latestVersion["releaseDate"] ?? latestVersion["release_date"] ?? latestVersion["date"] ?? latestVersion["published"];

          addOpeningUrlTo("open-client-release", "click", `https://codeberg.org/nullclyze/salarixi/releases/tag/${tag}`); 

          notice.style.display = "flex";
        }
      } catch (error) {
        logger.log(`Ошибка проверки обновлений (не критично): ${error}`, "error");
      }
    }, 4000);
  }

  /** Метод загрузки актуальной информации о последней версии клиента */
  private async loadNewVersionInfo(): Promise<any | null> {
    try {
      const latestVersion = await invoke("download_json", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.version.json",
      }) as any;

      if (!latestVersion) {
        logger.log(`Ошибка проверки обновлений (не критично): Resource moved or is unavailable`, "error");
        return;
      }

      return latestVersion;
    } catch (error) {
      logger.log(`Ошибка проверки обновлений (не критично): ${error}`, "error");
    }

    return null;
  }

  /** Метод загрузки кэшированной информации о последней версии клиента */
  private async loadCachedVersionInfo(): Promise<any | null> {
    const result = await loadCache("latest-version", "latest-version.json", 10, TimeType.Hours);
    if (!result.data) return null;
    return JSON.parse(result.data);
  }

  /** Метод кэширования информации о последней версии клиента */
  private async cacheVersionInfo(info: any): Promise<void> {
    const result = await saveCache(JSON.stringify(info, null, 2), "latest-version.json");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования информации о последней версии клиента (не критично): ${result.error}`, "error");
      return;
    }
  }
}

const updaterModule = new UpdaterModule();

export { updaterModule }
