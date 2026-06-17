import { invoke } from "@tauri-apps/api/core";

import { marked } from "marked";
import DOMPurify from "dompurify";

import { logger } from "../utils/logger";
import { loadCache, saveCache, TimeType } from "../utils/cache";
import { getCacheOrNew, LcfResult } from "../utils/cache_or_new";

/** Функция очистки HTML */
const sanitizeHtml = (html: string): string => DOMPurify.sanitize(html, {
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

/** Модуль управления пользовательской документацией */
class DocsModule {
  /** Метод инициализации модуля */
  public async init(): Promise<void> {
    try {
      logger.log("Загрузка документации...", "system");

      const docsWrapper = document.querySelector(".documentation-wrapper") as HTMLDivElement;
     
      const result = await getCacheOrNew<string>(this.lcf, this.lnf, this.scf);
      if (!result.data) return;

      docsWrapper.innerHTML = result.data;

      logger.log(`Документация загружена (размер ${result.size})`, "system");
    } catch (error) {
      logger.log(`Ошибка загрузки документации (не критично): ${error}`, "non-critical-error");
    }
  }

  /** Метод загрузки актуальной пользовательской документации */
  private async lnf(): Promise<string | null> {
    try {
      const docs = await invoke("download_text", {
        url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/docs/user.md"
      }) as string | null;

      if (!docs) {
        logger.log("Ошибка загрузки документации (не критично): Resource moved or is unavailable", "non-critical-error");
        return null;
      }

      const html = await marked.parse(docs);
      return sanitizeHtml(html);
    } catch (error) {
      logger.log(`Ошибка загрузки документации (не критично): ${error}`, "non-critical-error");
    }

    return null;
  }

  /** Метод загрузки кэшированной пользовательской документации */
  private async lcf(): Promise<LcfResult> {
    const result = await loadCache("user-docs", "user-docs.md", 2, TimeType.Day);
    if (!result.data) return { data: null, expired: false };

    const html = await marked.parse(result.data);
    const sanitized = sanitizeHtml(html);

    return {
      data: sanitized,
      expired: result.error === "expired",
    };
  }

  /** Метод кэширования пользовательской документации */
  private async scf(docs: string): Promise<void> {
    const result = await saveCache(docs, "user-docs.md");

    if (!result.data && result.error) {
      logger.log(`Ошибка кэширования документации (не критично): ${result.error}`, "non-critical-error");
      return;
    }
  }
}

const docsModule = new DocsModule();

export { docsModule }
