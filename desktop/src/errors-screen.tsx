import { useEffect, useRef } from "preact/hooks";
import { client, state } from "./api";
import { ApplicationError } from "./bindings";
import { Dialog, useDialog } from "./dialog";

const ErrorDialog = ({
  error,
  popup = false,
  onClose,
}: {
  error: ApplicationError;
  popup?: boolean;
  onClose?: () => void;
}) => {
  const { open, setOpen, innerRef } = useDialog<HTMLDivElement>();
  return (
    <Dialog
      modal={true}
      openText={error.type}
      open={popup ? error.read : open}
      setOpen={setOpen}
      onClose={onClose}
    >
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

export const ErrorsPopup = () => {
  const ref = useRef<HTMLDialogElement>(null);
  const unreadErrors = state.value.errors.filter((error) => !error.read);
  useEffect(
    () =>
      unreadErrors.length > 0
        ? ref?.current?.showModal()
        : ref.current?.close(),
    [unreadErrors.length > 0]
  );
  return (
    <dialog>
      {[
        unreadErrors
          .reverse()
          .map((error, index) => (
            <ErrorDialog
              error={error}
              popup={true}
              onClose={() => client.markErrorRead(index)}
            />
          )),
      ]}
    </dialog>
  );
};
