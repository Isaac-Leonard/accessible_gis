import { client } from "./api";
import { bindedSelectorFactory } from "./option-picker";

export const RenderMethodsSelector = bindedSelectorFactory(
  await client.getRenderMethods()
);
