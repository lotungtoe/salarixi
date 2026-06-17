import { getStringSize } from "./size";

/** Структура данных результата функции `getCacheOrNew` */
interface CacheOrNewResult<T> {
  data: T | null;
  size: string;
}

/** Структура данных результата функции загрузки кэша */
export interface LcfResult {
  data: any;
  expired: boolean;
}

/** Вспомогательная функция конвертации данных в строку */
const stringify = (data: any): string => typeof data === "object" ? JSON.stringify(data) : typeof data === "number" ? data.toString() : data as string;

/** Функция получения кэшированной или новой информации */
export async function getCacheOrNew<T>(lcf: Function, lnf: Function, scf: Function, onerr?: Function): Promise<CacheOrNewResult<T>> {
  const cache = await lcf() as LcfResult;
  let data = null;
  let dataSize = 0;

  if (cache.data && !cache.expired) {
    data = cache.data;
    dataSize = getStringSize(stringify(cache.data));
  } else {
    const newData = await lnf();

    if (newData) {
      data = newData;
      dataSize = getStringSize(stringify(newData));
      await scf(data);
    } else {
      if (cache.data) {
        data = cache.data;
        dataSize = getStringSize(stringify(cache.data));
      } else {
        if (onerr) onerr();

        return {
          data: null,
          size: "0KB",
        };
      }
    }
  }

  const prettySize = (dataSize / 1024).toFixed(1);

  return {
    data: data,
    size: `${prettySize}KB`,
  };
}
