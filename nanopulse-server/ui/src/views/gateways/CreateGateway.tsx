import { useNavigate } from "react-router-dom";
import { notifications } from "@mantine/notifications";

import { PageHeader } from "../../components/PageHeader";
import type { Workspace, Gateway } from "../../storage/types";
import { client } from "../../storage/client";
import { GatewayForm } from "./GatewayForm";
import { Paper } from "../../components/Paper";

export function CreateGateway(props: { workspace: Workspace }) {
  const navigate = useNavigate();

  const onSubmit = async (v: Gateway) => {
    const { error } = await client.POST(
      "/api/workspaces/{workspace_name}/gateways",
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
        title: "Gateway",
        message: "The gateway has been added",
        color: "green",
      });
      navigate(`/workspaces/${props.workspace.name}/gateways/${v.name}`);
    }
  };

  return (
    <>
      <PageHeader
        title="Add gateway"
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
          {
            label: "Add",
          },
        ]}
      />
      <Paper>
        <GatewayForm initialValues={{} as Gateway} onSubmit={onSubmit} />
      </Paper>
    </>
  );
}
