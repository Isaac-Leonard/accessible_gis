import { ApplicationError } from "./bindings";
import { Dialog, useDialog } from "./dialog";

const ErrorDialog = ({ error }: { error: ApplicationError }) => {
  const { open, setOpen, innerRef } = useDialog<HTMLDivElement>();
  return (
    <Dialog modal={true} openText={error.type} open={open} setOpen={setOpen}>
      <h1>{error.type}</h1>
      <div tabIndex={-1} ref={innerRef}>
        {error.type}
      </div>
    </Dialog>
  );
};

export const ErrorsScreen = ({ errors }: { errors: ApplicationError[] }) => {
  return (
    <div>
      <h2>Errors</h2>
      <div>
        <ul>
          {errors.map((error) => (
            <ErrorDialog error={error} />
          ))}
        </ul>
      </div>{" "}
    </div>
  );
};
