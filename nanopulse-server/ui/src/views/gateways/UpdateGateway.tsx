import { notifications } from "@mantine/notifications";

import type { Workspace, Gateway } from "../../storage/types";
import { client } from "../../storage/client";
import { GatewayForm } from "./GatewayForm";
import { Paper } from "../../components/Paper";

interface IProps {
  workspace: Workspace;
  gateway: Gateway;
}

export function UpdateGateway(props: IProps) {
  const onSubmit = async (v: Gateway) => {
    const { error } = await client.PUT(
      "/api/workspaces/{workspace_name}/gateways/{gateway_name}",
      {
        params: {
          path: {
            workspace_name: props.workspace.name,
            gateway_name: props.gateway.name,
          },
        },
        body: v,
      },
    );

    if (!error) {
      notifications.show({
        title: "Gateway",
        message: "The gateway has been updated",
        color: "green",
      });
    }
  };

  return (
    <Paper>
      <GatewayForm update initialValues={props.gateway} onSubmit={onSubmit} />
    </Paper>
  );
}
