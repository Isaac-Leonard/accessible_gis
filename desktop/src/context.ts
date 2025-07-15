import { createContext } from "preact";
import { ProjectScreenInfo } from "./bindings";

// Shouldn't really use null! here but don't want to write out a dummy object
export const LayerScreenContext = createContext<ProjectScreenInfo>(null!);
