import { EventEmitter } from "events";
import type { Workspace } from "./types";

class Session extends EventEmitter {
  workspace?: Workspace;

  constructor() {
    super();
  }

  setWorkspace = (w: Workspace) => {
    this.workspace = w;
    this.emit("workspace.set");
  };

  getWorkspace = (): Workspace | undefined => {
    return this.workspace;
  };
}

export const session = new Session();
