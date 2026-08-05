import { EventEmitter } from "events";
import { Websocket, WebsocketEvent } from "websocket-ts";

import type { WsEvent } from "./types";

class Events extends EventEmitter {
  socket?: Websocket;

  constructor() {
    super();
  }

  subscribeWorkspace = (name: string) => {
    if (this.socket) {
      this.socket.close();
    }

    this.socket = new Websocket("./api/ws");
    this.socket.addEventListener(WebsocketEvent.open, (ws) => {
      console.log("connected to server, workspace:", name);
      ws.send(
        JSON.stringify({
          workspace_name: name,
        }),
      );
    });

    this.socket.addEventListener(WebsocketEvent.message, (_ws, ev) => {
      console.log("ws notification:", ev.data);
      const n: WsEvent = JSON.parse(ev.data);
      this.emit("event", n);
    });
  };

  unsubscribe = () => {
    if (this.socket) {
      this.socket.close();
      this.socket = undefined;
    }
  };
}

export const events = new Events();
