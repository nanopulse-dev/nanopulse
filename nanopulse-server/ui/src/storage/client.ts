import createClient from "openapi-fetch";
import type { Middleware } from "openapi-fetch";
import { notifications } from "@mantine/notifications";
import { EventEmitter } from "events";

import type { paths } from "./schema";

class ClientEvents extends EventEmitter {}

export const clientEvents = new ClientEvents();

const eventMiddleware: Middleware = {
  async onResponse({ request, response }) {
    if (response.status >= 300) {
      const responseClone = response.clone();
      const data = await responseClone.json();

      console.log("api error: ", data);
      notifications.show({
        title: "Error",
        color: "red",
        message: data.message,
      });
    }

    const url = new URL(request.url);
    clientEvents.emit(`${request.method}:${url.pathname}`);

    return response;
  },

  onError({ error }) {
    console.error(error);
    notifications.show({
      title: "Error",
      color: "red",
      message: `${error}`,
    });
    return;
  },
};

export const client = createClient<paths>({ baseUrl: "./" });
client.use(eventMiddleware);
