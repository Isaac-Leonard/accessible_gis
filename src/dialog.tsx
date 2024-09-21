import { ComponentChildren } from "preact";
import { useEffect, useRef, useState } from "preact/hooks";

type DialogProps = {
  openText: string;
  modal?: boolean;
  open: boolean;
  setOpen: (open: boolean) => void;
  children: ComponentChildren;
};

export const Dialog = ({
  openText,
  modal,
  open,
  children,
  setOpen,
}: DialogProps) => {
  const ref = useRef<HTMLDialogElement | null>(null);
  const closeRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    if (open) {
      if (modal ?? true) {
        ref.current?.showModal();
      } else {
        ref.current?.show();
      }
    } else {
      ref.current?.close();
    }
  }, [open, modal]);
  useEffect(() => {
    if (ref.current?.open === false) {
      closeRef.current?.focus();
    }
  }, [ref.current?.open]);

  return (
    <div>
      <button ref={closeRef} onClick={() => setOpen(true)}>
        {openText}
      </button>
      <dialog onClose={() => setOpen(false)} ref={ref}>
        {children}
      </dialog>
    </div>
  );
};

export const useDialog = <T extends HTMLElement>() => {
  const [open, setOpen] = useState(false);
  const innerRef = useRef<T>(null);
  useEffect(() => {
    if (open) {
      innerRef.current?.focus();
    }
  }, [open]);
  return { open, setOpen, innerRef };
};
