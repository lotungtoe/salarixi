import { getStringSize } from "./size";

/** Структура данных результата функции `getCacheOrNew` */
interface CacheOrNewResult<T> {
  data: T | null;
  size: string;
}

/** Вспомогательная функция конвертации данных в строку */
const stringify = (data: any): string => typeof data === "object" ? JSON.stringify(data) : typeof data === "number" ? data.toString() : data as string;

/** Функция получения кэшированной или новой информации */
export async function getCacheOrNew<T>(lcf: Function, lnf: Function, scf: Function, onerr?: Function): Promise<CacheOrNewResult<T>> {
  const cachedData = await lcf();
  let data = null;
  let dataSize = 0;

  if (cachedData) {
    data = cachedData;
    dataSize = getStringSize(stringify(cachedData));
  } else {
    const newData = await lnf();

    if (newData) {
      data = newData;
      dataSize = getStringSize(stringify(newData));
      await scf(data);
    } else {
      if (!cachedData["data"] && !cachedData["error"]) {
        data = cachedData;
        dataSize = getStringSize(stringify(cachedData));
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
