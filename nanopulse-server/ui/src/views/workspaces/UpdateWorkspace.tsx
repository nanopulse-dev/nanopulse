import { notifications } from "@mantine/notifications";

import type { Workspace } from "../../storage/types";
import { client } from "../../storage/client";
import { WorkspaceForm } from "./WorkspaceForm";
import { Paper } from "../../components/Paper";

export function UpdateWorkspace(props: { workspace: Workspace }) {
  const onSubmit = async (w: Workspace) => {
    const { error } = await client.PUT("/api/workspaces/{workspace_name}", {
      params: {
        path: {
          workspace_name: props.workspace.name,
        },
      },
      body: w,
    });

    if (!error) {
      notifications.show({
        title: "Workspace",
        message: "The workspace has been updated",
        color: "green",
      });
    }
  };

  return (
    <Paper>
      <WorkspaceForm
        update
        initialValues={props.workspace}
        onSubmit={onSubmit}
      />
    </Paper>
  );
}
