import { useEffect, useState, useCallback } from "react";
import { useParams } from "react-router";
import { useNavigate } from "react-router";

import { Button, Tabs, Group, ActionIcon } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { IconRefresh } from "@tabler/icons-react";

import type { Device, Workspace } from "../../storage/types";
import { client } from "../../storage/client";
import { PageHeader } from "../../components/PageHeader";
import { Paper } from "../../components/Paper";
import { UpdateDevice } from "./UpdateDevice";
import { DeviceDashboard } from "./DeviceDashboard";

export function DeviceLoader(props: { workspace: Workspace }) {
  const navigate = useNavigate();
  const { deviceName } = useParams();
  const [device, setDevice] = useState<Device | null>(null);

  const getDevice = useCallback(async () => {
    if (deviceName) {
      const { data } = await client.GET(
        "/api/workspaces/{workspace_name}/devices/{device_name}",
        {
          params: {
            path: {
              workspace_name: props.workspace.name,
              device_name: deviceName,
            },
          },
        },
      );

      if (data) {
        setDevice(data);
      }
    }
  }, [props.workspace, deviceName]);

  useEffect(() => {
    getDevice();
  }, [getDevice]);

  const deleteDevice = async () => {
    if (
      deviceName &&
      confirm("Are you sure you would like to delete this device?")
    ) {
      const { error } = await client.DELETE(
        "/api/workspaces/{workspace_name}/devices/{device_name}",
        {
          params: {
            path: {
              workspace_name: props.workspace.name,
              device_name: deviceName,
            },
          },
        },
      );

      if (!error) {
        notifications.show({
          title: "Device deleted",
          message: "The device has been deleted",
          color: "orange",
        });

        navigate(`/workspaces/${props.workspace.name}/devices`);
      }
    }
  };

  if (!device) {
    return null;
  }

  return (
    <>
      <PageHeader
        title={device.name}
        breadcrumb={[
          { label: "Workspaces" },
          {
            label: props.workspace.name,
            link: `/workspaces/${props.workspace.name}`,
          },
          {
            label: "Devices",
            link: `/workspaces/${props.workspace.name}/devices`,
          },
          { label: device.name },
        ]}
        extra={
          <Group>
            <ActionIcon size="lg" variant="outline" onClick={getDevice}>
              <IconRefresh />
            </ActionIcon>
            <Button color="red" onClick={deleteDevice}>
              Delete device
            </Button>
          </Group>
        }
      />
      <Paper>
        <Tabs defaultValue="dashboard">
          <Tabs.List>
            <Tabs.Tab value="dashboard">Dashboard</Tabs.Tab>
            <Tabs.Tab value="config">Configuration</Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel value="dashboard">
            <Paper>
              <DeviceDashboard workspace={props.workspace} device={device} />
            </Paper>
          </Tabs.Panel>
          <Tabs.Panel value="config">
            <UpdateDevice workspace={props.workspace} device={device} />
          </Tabs.Panel>
        </Tabs>
      </Paper>
    </>
  );
}
