import { signal } from "@preact/signals";
import { Result } from "./bindings";
import { commands } from "./bindings";

export const state = signal(await commands.getAppInfo());
type Api = typeof commands;
type ExtractData<Api> = {
  [K in keyof Api]: Api[K] extends Promise<Result<infer Data, infer Err>>
    ? Promise<Data>
    : Api[K];
};

export const client: ExtractData<Api> = Object.entries(commands).reduce(
  (client, [key, fn]) => {
    const func = async <T extends []>(...args: T) => {
      const res = await fn(...args);
      if (res?.status === "ok") {
        return res.data;
      } else if (res?.status === "error") {
        throw res.error;
      } else {
        return res;
      }
    };
    if (!key.startsWith("get")) {
      return {
        ...client,
        [key]: (...args: Parameters<typeof fn>) => {
          const res = func(...args);
          rerender();
          return res;
        },
      };
    } else {
      return { ...client, [key]: func };
    }
  },
  {}
);

export const rerender = () => {
  commands.getAppInfo().then((data) => {
    state.value = data;
  });
};
