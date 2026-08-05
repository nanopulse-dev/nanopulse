import { useNavigate } from "react-router";

import { Button, Tabs } from "@mantine/core";
import { notifications } from "@mantine/notifications";

import type { Workspace } from "../../storage/types";
import { PageHeader } from "../../components/PageHeader";
import { client } from "../../storage/client";
import { Paper } from "../../components/Paper";
import { UpdateWorkspace } from "./UpdateWorkspace";

export function WorkspaceDashboard(props: { workspace: Workspace }) {
  const navigate = useNavigate();

  const deleteWorkspace = async () => {
    if (confirm("Are you sure you would like to delete this workspace?")) {
      const { error } = await client.DELETE(
        "/api/workspaces/{workspace_name}",
        {
          params: {
            path: {
              workspace_name: props.workspace.name,
            },
          },
        },
      );

      if (!error) {
        notifications.show({
          title: "Workspace deleted",
          message: "The workspace has been deleted",
          color: "orange",
        });

        navigate("/");
      }
    }
  };

  return (
    <>
      <PageHeader
        title={props.workspace.name}
        breadcrumb={[{ label: "Workspaces" }, { label: props.workspace.name }]}
        extra={
          <Button color="red" onClick={deleteWorkspace}>
            Delete workspace
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
            <UpdateWorkspace workspace={props.workspace} />
          </Tabs.Panel>
        </Tabs>
      </Paper>
    </>
  );
}
