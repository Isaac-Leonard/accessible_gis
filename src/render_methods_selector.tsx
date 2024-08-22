import { client } from "./api";
import { RenderMethod } from "./bindings";
import { OptionPicker } from "./option-picker";

const options = await client.getRenderMethods();

type RenderMethodsSelectorProps = {
  selectedMethod: RenderMethod;
  setMethod: (method: RenderMethod) => void;
};

export const RenderMethodsSelector = ({
  selectedMethod,
  setMethod,
}: RenderMethodsSelectorProps) => {
  return (
    <div>
      <OptionPicker
        selectedOption={selectedMethod}
        options={options}
        setOption={setMethod}
        prompt="Ways to visually render raster data"
        emptyText="Something has gone wrong"
      />
    </div>
  );
};
