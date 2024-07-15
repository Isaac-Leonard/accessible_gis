import { ComponentChildren } from "preact";
import { useEffect, useRef, useState } from "preact/hooks";

type DrawerProps = {
  children: ComponentChildren;
  openText: string;
  open: boolean;
  setOpen: (open: boolean) => void;
};

export const Drawer = ({ children, openText, open, setOpen }: DrawerProps) => {
  const buttonRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!open) {
      buttonRef.current?.focus();
    }
  }, [open]);
  if (open) {
    return (
      <div
        tabIndex={-1}
        role="dialog"
        onKeyDown={(e) => {
          console.log(e.key);
          if (e.key === "Escape") {
            e.preventDefault();
            setOpen(false);
          }
        }}
      >
        {children}
      </div>
    );
  } else {
    return (
      <div>
        <button ref={buttonRef} onClick={() => setOpen(true)}>
          {openText}
        </button>
      </div>
    );
  }
};

export const useDrawer = <T extends HTMLElement>() => {
  const [open, setOpen] = useState(false);
  const innerRef = useRef<T>(null);
  useEffect(() => {
    if (open) {
      innerRef.current?.focus();
    }
  }, [open]);
  return { open, setOpen, innerRef };
};
