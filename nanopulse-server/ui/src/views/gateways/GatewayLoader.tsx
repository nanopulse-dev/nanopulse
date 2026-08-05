import { useEffect, useState, useCallback } from "react";
import { useParams } from "react-router";
import { useNavigate } from "react-router";

import { Button, Tabs } from "@mantine/core";
import { notifications } from "@mantine/notifications";

import type { Gateway, Workspace } from "../../storage/types";
import { client } from "../../storage/client";
import { PageHeader } from "../../components/PageHeader";
import { Paper } from "../../components/Paper";
import { clientEvents } from "../../storage/client";

import { UpdateGateway } from "./UpdateGateway";

export function GatewayLoader(props: { workspace: Workspace }) {
  const navigate = useNavigate();
  const { gatewayName } = useParams();
  const [gateway, setGateway] = useState<Gateway | null>(null);

  const getGateway = useCallback(async () => {
    if (gatewayName) {
      const { data } = await client.GET(
        "/api/workspaces/{workspace_name}/gateways/{gateway_name}",
        {
          params: {
            path: {
              workspace_name: props.workspace.name,
              gateway_name: gatewayName,
            },
          },
        },
      );

      if (data) {
        setGateway(data);
      }
    }
  }, [props.workspace, gatewayName]);

  useEffect(() => {
    if (gatewayName) {
      getGateway();
      console.log("BOO");

      clientEvents.on(
        `PUT:/api/workspaces/${props.workspace.name}/gateways/${gatewayName}`,
        getGateway,
      );

      return () => {
        clientEvents.removeListener(
          `PUT:/api/workspaces/${props.workspace.name}/gateways/${gatewayName}`,
          getGateway,
        );
      };
    }
  }, [getGateway]);

  const deleteGateway = async () => {
    if (
      gatewayName &&
      confirm("Are you sure you would like to delete this gateway?")
    ) {
      const { error } = await client.DELETE(
        "/api/workspaces/{workspace_name}/gateways/{gateway_name}",
        {
          params: {
            path: {
              workspace_name: props.workspace.name,
              gateway_name: gatewayName,
            },
          },
        },
      );

      if (!error) {
        notifications.show({
          title: "Gateway deleted",
          message: "The gateway has been deleted",
          color: "orange",
        });

        navigate(`/workspaces/${props.workspace.name}/gateways`);
      }
    }
  };

  if (!gateway) {
    return null;
  }

  return (
    <>
      <PageHeader
        title={gateway.name}
        breadcrumb={[
          { label: "Workspaces" },
          {
            label: props.workspace.name,
            link: `/workspaces/${props.workspace.name}`,
          },
          {
            label: "Gateways",
            link: `/workspaces/${props.workspace.name}/gateways`,
          },
          { label: gateway.name },
        ]}
        extra={
          <Button color="red" onClick={deleteGateway}>
            Delete gateway
          </Button>
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
              <div>TODO</div>
            </Paper>
          </Tabs.Panel>
          <Tabs.Panel value="config">
            <UpdateGateway gateway={gateway} workspace={props.workspace} />
          </Tabs.Panel>
        </Tabs>
      </Paper>
    </>
  );
}
