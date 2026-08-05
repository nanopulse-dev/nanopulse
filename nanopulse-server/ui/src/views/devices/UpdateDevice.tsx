import { notifications } from "@mantine/notifications";

import type { Workspace, Device } from "../../storage/types";
import { client } from "../../storage/client";
import { Paper } from "../../components/Paper";
import { DeviceForm } from "./DeviceForm";

interface IProps {
  workspace: Workspace;
  device: Device;
}

export function UpdateDevice(props: IProps) {
  const onSubmit = async (v: Device) => {
    const { error } = await client.PUT(
      "/api/workspaces/{workspace_name}/devices/{device_name}",
      {
        params: {
          path: {
            workspace_name: props.workspace.name,
            device_name: props.device.name,
          },
        },
        body: v,
      },
    );

    if (!error) {
      notifications.show({
        title: "Device",
        message: "The device has been updated",
        color: "green",
      });
    }
  };

  return (
    <Paper>
      <DeviceForm update initialValues={props.device} onSubmit={onSubmit} />
    </Paper>
  );
}
