import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { loadCache, saveCache, TimeType } from "../utils/cache";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { getStringSize } from "../utils/size";

/** Модуль управления пользовательской документацией */
class DocsModule {
  /** Метод инициализации модуля */
  public async init(): Promise<void> {
    try {
      logger.log("Загрузка документации...", "system");

      const docsEl = document.querySelector(".documentation-wrapper") as HTMLDivElement;
      const cachedDocs = await this.loadCachedUserDocs();
      let dataSize = 0;

      if (cachedDocs) {
        docsEl.innerHTML = cachedDocs;
        dataSize = getStringSize(cachedDocs);
      } else {
        const newDocs = await this.loadNewUserDocs();

        if (newDocs) {
          docsEl.innerHTML = newDocs;
          dataSize = getStringSize(newDocs);
          await this.cacheUserDocs(newDocs);
        } else {
          return;
        }
      }

      const prettyDataSize = (dataSize / 1024).toFixed(1);

      logger.log(`Документация загружена (размер ${prettyDataSize}KB)`, "system");
    } catch (error) {
      logger.log(`Ошибка загрузки документации (не критично): ${error}`, "error");
    }
  }

  /** Вспомогательный метод обработки HTML, чтобы избежать XSS уязвимости */
  private sanitizeHtml(html: string): string {
    return DOMPurify.sanitize(html, {
      ALLOWED_TAGS: [
        "b", "i", "em", "strong", "small", "mark", "del", "ins", "s", "sub", "sup",
        "u", "span", "abbr", "cite", "dfn", "kbd", "samp", "var", "h1", "h2", "p",
        "h3", "h4", "h5", "h6", "ul", "ol", "li", "dl", "dt", "dd", "table", "thead",
        "tbody", "tfoot", "tr", "th", "td", "caption", "col", "colgroup", "a", "img",
        "pre", "code", "section", "article", "header", "footer", "main", "aside", "nav", 
        "figure", "figcaption", "details", "summary", "fieldset", "legend", "label"
      ],
      ALLOWED_ATTR: [],
      ALLOW_DATA_ATTR: false,
    });
  }

  /** Метод загрузки актуальной пользовательской документации */
  private async loadNewUserDocs(): Promise<string | null> {
    try {
      const docs = await invoke("download_text", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/docs/user.md"
      }) as string | null;

      if (!docs) {
        logger.log("Ошибка загрузки документации (не критично): Resource moved or is unavailable", "error");
        return null;
      }

      const html = await marked.parse(docs);
      return this.sanitizeHtml(html);
    } catch (error) {
      logger.log(`Ошибка загрузки документации (не критично): ${error}`, "error");
    }

    return null;
  }

  /** Метод загрузки кэшированной пользовательской документации */
  private async loadCachedUserDocs(): Promise<string | null> {
    const result = await loadCache("user-docs", "user-docs.md", 2, TimeType.Day);
    if (!result.data) return null;

    const html = await marked.parse(result.data);
    return this.sanitizeHtml(html);
  }

  /** Метод кэширования пользовательской документации */
  private async cacheUserDocs(docs: string): Promise<void> {
    const result = await saveCache(docs, "user-docs.md");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования документации (не критично): ${result.error}`, "error");
      return;
    }
  }
}

const docsModule = new DocsModule();

export { docsModule }
