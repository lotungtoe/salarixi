import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { addOpeningUrlTo } from "../main";
import { isVersionNewer } from "../utils/semver";
import { loadCacheOrNew, LcfResult, LnfResult, loadCache, saveCache, TimeType } from "../utils/cache";

/** Небольшой модуль управления обновлениями клиента */
class UpdaterModule {
  /** Метод инициализации модуля */
  public init(): void {
    setTimeout(async () => {
      try {
        const notice = document.getElementById("notice") as HTMLElement;

        document.getElementById("close-notice-btn")?.addEventListener("click", () => notice.remove());

        const onerr = (err: string) => {
          notice.remove();
          logger.log(`Ошибка проверки обновлений (не критично): ${err}`, "non-critical-error");
        };

        const result = await loadCacheOrNew<any>(this.lcf, this.lnf, this.scf, onerr);
        if (!result.data) return;

        const data = result.data;
        const version = data["version"] ?? data["id"] ?? data["version_id"] ?? data["name"];
        const url = data["url"] ?? data["download_url"] ?? data["release_url"] ?? data["target_url"];
        const tag = data["tag"] ?? data["version_tag"] ?? data["release_tag"];
        const published = data["published"] ?? data["released"] ?? data["release_date"] ?? data["date"];

        if (!isVersionNewer(version)) {
          notice.remove();
          return;
        }

        (document.getElementById("new-client-version") as HTMLElement).innerText = version;
        (document.getElementById("new-client-tag") as HTMLElement).innerText = tag;
        (document.getElementById("new-client-published") as HTMLElement).innerText = published;

        const releaseUrl = url ? url : `https://codeberg.org/nullclyze/salarixi/releases/tag/${tag}`;
        addOpeningUrlTo("open-client-release", "click", releaseUrl); 

        notice.style.display = "flex";
      } catch (error) {
        logger.log(`Ошибка проверки обновлений (не критично): ${error}`, "non-critical-error");
      }
    }, 4000);
  }

  /** Метод загрузки актуальной информации о последней версии клиента */
  private async lnf(): Promise<LnfResult> {
    try {
      const version = await invoke("download_json", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.version.json",
      }) as any;

      return { data: version, error: version ? null : "resource moved or unavailable" };
    } catch (err) {
      return { data: null, error: String(err) };
    }
  }

  /** Метод загрузки кэшированной информации о последней версии клиента */
  private async lcf(): Promise<LcfResult> {
    const result = await loadCache("latest-version", "latest-version.json", 14, TimeType.Hours);
    if (!result.data) return { data: null, incompatible: false, expired: false };

    return {
      data: JSON.parse(result.data),
      incompatible: result.error === "incompatible",
      expired: result.error === "expired",
    };
  }

  /** Метод кэширования информации о последней версии клиента */
  private async scf(info: any): Promise<void> {
    const result = await saveCache(JSON.stringify(info, null, 2), "latest-version.json");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования информации о последней версии клиента (не критично): ${result.error}`, "non-critical-error");
    }
  }
}

const updaterModule = new UpdaterModule();

export { updaterModule }
