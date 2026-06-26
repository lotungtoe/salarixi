import { logger } from "../utils/logger";
import { plugins } from "../common/structs";
import { process } from "../main";
import { invoke } from "@tauri-apps/api/core";
import { loadCache, saveCache, TimeType } from "../utils/cache";
import { getCacheOrNew, LcfResult } from "../utils/cache_or_new";

/** Модуль загрузки и управления плагинами */
class PluginModule {
  /** Метод инициализации функций, связанных с плагинами */
  public async init(): Promise<void> {
    this.createCards();
    for (const id in plugins) this.updatePluginState(id, localStorage.getItem(`plugin-state:${id}`) === "true");
    await this.initPluginsInfo();
  }

  /** Метод обновления и сохранения состояния плагина (включен / выключен) */
  private updatePluginState(id: string, state: boolean): void {
    if (process !== "sleep") {
      logger.log(`Не удалось изменить состояние плагина ${id}: Plugin states can only be changed before startup`, "warning");
      return;
    }

    plugins[id].enable = state;

    localStorage.setItem(`plugin-state:${id}`, String(state));

    const toggler = document.getElementById(`${id}-toggler`) as HTMLButtonElement;

    if (state) {
      toggler.setAttribute("state", "false");
      toggler.style.color = "#4ed618";
      toggler.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-check-lg" viewBox="0 0 16 16">
          <path d="M12.736 3.97a.733.733 0 0 1 1.047 0c.286.289.29.756.01 1.05L7.88 12.01a.733.733 0 0 1-1.065.02L3.217 8.384a.757.757 0 0 1 0-1.06.733.733 0 0 1 1.047 0l3.052 3.093 5.4-6.425z"/>
        </svg>
      `;
    } else {
      toggler.setAttribute("state", "true");
      toggler.style.color = "#e61919";
      toggler.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
          <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
        </svg>
      `;
    }
  }

  /** Метод создания и инициализации карточек плагинов */
  private createCards(): void {
    for (const id in plugins) {
      try {
        const plugin = plugins[id];

        const pluginCard = document.createElement("div");
        pluginCard.className = "plugin-card";
        pluginCard.id = `${id}-plugin`;

        pluginCard.innerHTML = `
          <label class="name" id="${id}-name">${plugin.name}</label>

          <div class="controls">
            <button plugin-open-description="true" path="${id}-plugin-description">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
                <path fill-rule="evenodd" d="M4.475 5.458c-.284 0-.514-.237-.47-.517C4.28 3.24 5.576 2 7.825 2c2.25 0 3.767 1.36 3.767 3.215 0 1.344-.665 2.288-1.79 2.973-1.1.659-1.414 1.118-1.414 2.01v.03a.5.5 0 0 1-.5.5h-.77a.5.5 0 0 1-.5-.495l-.003-.2c-.043-1.221.477-2.001 1.645-2.712 1.03-.632 1.397-1.135 1.397-2.028 0-.979-.758-1.698-1.926-1.698-1.009 0-1.71.529-1.938 1.402-.066.254-.278.461-.54.461h-.777ZM7.496 14c.622 0 1.095-.474 1.095-1.09 0-.618-.473-1.092-1.095-1.092-.606 0-1.087.474-1.087 1.091S6.89 14 7.496 14"/>
              </svg>
            </button>

            <button plugin-toggler="true" plugin-id="${id}" state="true" id="${id}-toggler" style="color: #d61818;">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16">
                <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
              </svg>
            </button>
          </div>
        `;

        document.getElementById("plugin-list")?.appendChild(pluginCard);
        
        const pluginDescription = document.createElement("div");
        pluginDescription.className = "cover";
        pluginDescription.id = `${id}-plugin-description`;

        pluginDescription.innerHTML = `
          <div class="panel with-header">
            <div class="left">
              <div class="header">Описание плагина "${plugin.name}"</div>
            </div>

            <div class="right">
              <button class="min" plugin-close-description="true" path="${id}-plugin-description">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-lg" viewBox="0 0 16 16">
                  <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                </svg>
              </button>
            </div>
          </div>

          <div class="plugin-description" id="${id}-description"></div>
          <p class="plugin-latest-update">Последнее обновление плагина: <span id="${id}-latest-update">?</span></p>
        `;

        document.getElementById("plugins-container")?.appendChild(pluginDescription);
      } catch (error) {
        logger.log(`Ошибка инициализации плагина ${id}: ${error}`, "error");
      }
    }

    document.querySelectorAll('[plugin-toggler="true"]').forEach(t => t.addEventListener("click", () => {
      const id = t.getAttribute("plugin-id") || "";
      const state = t.getAttribute("state") === "true";
      this.updatePluginState(id, state);
    }));  

    document.querySelectorAll('[plugin-open-description="true"]').forEach(e => e.addEventListener("click", () => {
      const path = e.getAttribute("path");
      if (!path) return;
      (document.getElementById(path) as HTMLElement).style.display = "flex";
    })); 

    document.querySelectorAll('[plugin-close-description="true"]').forEach(e => e.addEventListener("click", () => {
      const path = e.getAttribute("path");
      if (!path) return;
      (document.getElementById(path) as HTMLElement).style.display = "none";
    }));
  }

  /** Метод инициализации информации о плагинах */
  private async initPluginsInfo(): Promise<void> {
    try {
      logger.log("Загрузка информации о плагинах...", "system");

      const result = await getCacheOrNew<any>(this.lcf, this.lnf, this.scf);
      if (!result.data) return;

      const data = result.data;
      const pluginList = data["list"] ?? data["array"] ?? data["map"] ?? data;

      // Может быть такое что названия полей изменились, или вовсе вся структура
      // данных обновлена, поэтому здесь стоит делать ещё одну проверку
      if (!pluginList) {
        logger.log("Ошибка загрузки информации о плагинах (не критично): Information is corrupted or missing", "non-critical-error");
        return;
      }

      this.extractPluginsInfo(pluginList);

      logger.log(`Информация о плагинах загружена (размер ${result.size})`, "system");
    } catch (error) {
      logger.log(`Ошибка загрузки информации о плагинах (не критично): ${error}`, "non-critical-error");
    }
  }

  /** Метод извлечения информации плагинов */
  private extractPluginsInfo(pluginList: any): void {
    for (const plugin of pluginList) {
      const id = plugin["id"] ?? plugin["name"] ?? plugin["header"] ?? plugin["kind"];
      const desc = plugin["description"] ?? plugin["purpose"];
      const updated = plugin["latest-update"] ?? plugin["updated"] ?? plugin["modified"];

      if (plugins[id]?.date) {
        if (updated !== plugins[id].date) {
          document.getElementById(`${id}-plugin`)?.classList.add("deprecated");

          const tag = document.createElement("span");
          tag.className = "tag";
          tag.innerText = "deprecated";

          document.getElementById(`${id}-name`)?.appendChild(tag);
        }

        const pluginDescriptionContainer = document.getElementById(`${id}-description`) as HTMLElement;
        const pluginLatestUpdateContainer = document.getElementById(`${id}-latest-update`) as HTMLElement;

        for (const p of desc) {
          const el = document.createElement("p");
          el.innerText = p;
          pluginDescriptionContainer.appendChild(el);
        }

        pluginLatestUpdateContainer.innerText = updated;
      }
    }
  }

  /** Метод загрузки актуальной информации о плагинах */
  private async lnf(): Promise<any> {
    try {
      const plugins = await invoke("download_json", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.plugins.json"
      }) as any;
      
      if (!plugins) {
        logger.log("Ошибка загрузки информации о плагинах (не критично): Resource moved or is unavailable", "non-critical-error");
        return;
      }

      return plugins;
    } catch (error) {
      logger.log(`Ошибка загрузки информации о плагинах (не критично): ${error}`, "non-critical-error");
    }

    return null;
  }

  /** Метод загрузки кэшированной информации о плагинах */
  private async lcf(): Promise<LcfResult> {
    const result = await loadCache("plugins-info", "plugins-info.json", 4, TimeType.Day);
    if (!result.data) return { data: null, expired: false };

    return {
      data: JSON.parse(result.data),
      expired: result.error === "expired",
    };
  }

  /** Метод кэширования информации о плагинах */
  private async scf(info: string): Promise<void> {
    const result = await saveCache(JSON.stringify(info, null, 2), "plugins-info.json");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования информации о плагинах (не критично): ${result.error}`, "non-critical-error");
      return;
    }
  }
}

const pluginModule = new PluginModule();

export { pluginModule }
