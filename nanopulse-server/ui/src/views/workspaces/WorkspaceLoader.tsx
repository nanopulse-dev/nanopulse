import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { Routes, Route } from "react-router";
import { Text, Modal, Button, TextInput } from "@mantine/core";
import { DataList, Group } from "@mantine/core";
import { useForm } from "@mantine/form";
import { notifications } from "@mantine/notifications";

import { session } from "../../storage/session";
import { client } from "../../storage/client";
import type {
  Workspace,
  Device,
  WsEvent,
  KeyExchange,
} from "../../storage/types";

import { WorkspaceDashboard } from "./WorkspaceDashboard";
import { ListGateways } from "../gateways/ListGateways";
import { ListDevices } from "../devices/ListDevices";
import { CreateGateway } from "../gateways/CreateGateway";
import { GatewayLoader } from "../gateways/GatewayLoader";
import { DeviceLoader } from "../devices/DeviceLoader";
import { events } from "../../storage/events";
import { Logs } from "../Logs";
import { Console } from "../Console";

interface KeyExchangeModalProps {
  workspace: Workspace;
  notification: KeyExchange;
  onClose: () => void;
}

function KeyExchangeModal(props: KeyExchangeModalProps) {
  const onSubmit = async (v: Device) => {
    v.public_key = props.notification.device_public_key;

    const { error } = await client.POST(
      "/api/workspaces/{workspace_name}/devices",
      {
        params: {
          path: {
            workspace_name: props.workspace.name,
          },
        },
        body: v,
      },
    );

    if (!error) {
      notifications.show({
        title: "Device",
        message: "The device has been added",
        color: "green",
      });
    }

    props.onClose();
  };

  const device = {} as Device;

  const form = useForm({
    mode: "uncontrolled",
    initialValues: device,
    validate: {
      pin: (value) => (/^[a-fA-F0-9]{8}$/.test(value) ? null : "Invalid PIN"),
      name: (value) =>
        /^[a-z0-9\-]*$/.test(value) ? null : "Enter a valid name",
    },
  });

  return (
    <Modal
      opened={props.notification.device_public_key !== ""}
      title={`New device: ${props.notification.device_short_id}`}
      onClose={props.onClose}
      centered
      size={680}
    >
      <Text size="sm">
        A new device is detected and tries to perform a key exchange. Please
        confirm its details and eter the PIN to accept the key exchange.
      </Text>
      <DataList orientation="vertical" mt="md">
        <DataList.Item key="shortId">
          <DataList.ItemLabel>Short ID</DataList.ItemLabel>
          <DataList.ItemValue>
            <pre>{props.notification.device_short_id}</pre>
          </DataList.ItemValue>
        </DataList.Item>
        <DataList.Item key="publicKey">
          <DataList.ItemLabel>Public Key</DataList.ItemLabel>
          <DataList.ItemValue>
            <pre>{props.notification.device_public_key}</pre>
          </DataList.ItemValue>
        </DataList.Item>
      </DataList>
      <form onSubmit={form.onSubmit((v) => onSubmit(v))}>
        <TextInput
          mt="md"
          label="PIN"
          key={form.key("pin")}
          {...form.getInputProps("pin")}
        />
        <TextInput
          mt="md"
          label="Device Name"
          key={form.key("name")}
          {...form.getInputProps("name")}
        />
        <Group justify="flex-end" mt="md">
          <Button variant="default" onClick={props.onClose}>
            Ignore
          </Button>
          <Button type="submit">Accept</Button>
        </Group>
      </form>
    </Modal>
  );
}

export function WorkspaceLoader() {
  const { workspaceName } = useParams();

  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [keyExchangeNotifications, setKeyExchangeNotifications] = useState<
    KeyExchange[]
  >([]);

  const onWsNotification = (n: WsEvent) => {
    if (n.key_exchange_event) {
      setKeyExchangeNotifications([
        ...keyExchangeNotifications,
        n.key_exchange_event,
      ]);
    }
  };

  useEffect(() => {
    if (workspaceName) {
      const fetch = async () => {
        const { data } = await client.GET("/api/workspaces/{workspace_name}", {
          params: {
            path: { workspace_name: workspaceName },
          },
        });

        if (data) {
          setWorkspace(data);
          session.setWorkspace(data);
        }
      };
      fetch();

      events.subscribeWorkspace(workspaceName);
      events.on("event", onWsNotification);

      return () => {
        events.removeListener("event", onWsNotification);
      };
    }
  }, [workspaceName]);

  const onCloseKeyExchangeNotification = (public_key: string): (() => void) => {
    return () => {
      const n = keyExchangeNotifications.filter(
        (v: KeyExchange) => v.device_public_key != public_key,
      );
      setKeyExchangeNotifications(n);
    };
  };

  const keyExchangeNotification: KeyExchange =
    keyExchangeNotifications.length > 0
      ? keyExchangeNotifications[0]
      : {
          timestamp: "",
          device_public_key: "",
          device_short_id: "",
          workspace_name: "",
        };

  if (!workspace) {
    return <></>;
  }

  return (
    <>
      <KeyExchangeModal
        workspace={workspace}
        notification={keyExchangeNotification}
        onClose={onCloseKeyExchangeNotification(
          keyExchangeNotification.device_public_key,
        )}
      />
      <Routes>
        <Route
          path="/"
          element={<WorkspaceDashboard workspace={workspace} />}
        />
        <Route
          path="/gateways"
          element={<ListGateways workspace={workspace} />}
        />
        <Route
          path="/gateways/:gatewayName"
          element={<GatewayLoader workspace={workspace} />}
        />
        <Route
          path="/gateways/create"
          element={<CreateGateway workspace={workspace} />}
        />
        <Route
          path="/devices"
          element={<ListDevices workspace={workspace} />}
        />
        <Route
          path="/devices/:deviceName"
          element={<DeviceLoader workspace={workspace} />}
        />
        <Route path="/logs" element={<Logs workspace={workspace} />} />
        <Route path="/console" element={<Console workspace={workspace} />} />
      </Routes>
    </>
  );
}
