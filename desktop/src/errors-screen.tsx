import { useEffect, useRef } from "preact/hooks";
import { client, state } from "./api";
import { ApplicationError } from "./bindings";
import { Dialog, useDialog } from "./dialog";

const ErrorDialog = ({
  error,
  onClose,
}: {
  error: ApplicationError;
  onClose?: () => void;
}) => {
  const { open, setOpen, innerRef } = useDialog<HTMLHeadingElement>();
  return (
    <Dialog
      modal={true}
      openText={error.type}
      open={!error.read || open}
      setOpen={setOpen}
      onClose={onClose}
    >
      <h1 ref={innerRef}>{error.type}</h1>
      <div>{JSON.stringify(error.error)}</div>
      {onClose && <button onClick={onClose}>Close</button>}
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
            <ErrorDialog key={error.id} error={error} />
          ))}
        </ul>
      </div>{" "}
    </div>
  );
};

export const ErrorsPopup = () => {
  const unreadErrors = state.value.errors.filter((error) => !error.read);
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (unreadErrors.length > 0) {
      ref?.current?.showModal();
      ref.current?.focus();
    } else {
      ref.current?.close();
    }
  }, [unreadErrors.length]);
  return (
    <dialog ref={ref}>
      {[
        unreadErrors.map((error) => (
          <ErrorDialog
            key={error.id}
            error={error}
            onClose={() => client.markErrorRead(error.id)}
          />
        )),
      ]}
    </dialog>
  );
};
