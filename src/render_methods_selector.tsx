import { client } from "./api";
import { selectorFactory } from "./option-picker";

export const RenderMethodsSelector = selectorFactory(
  await client.getRenderMethods()
);
