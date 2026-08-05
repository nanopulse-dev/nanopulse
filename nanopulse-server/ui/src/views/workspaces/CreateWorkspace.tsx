import { useNavigate } from "react-router-dom";
import { notifications } from "@mantine/notifications";

import { PageHeader } from "../../components/PageHeader";
import { WorkspaceForm } from "./WorkspaceForm";
import type { Workspace } from "../../storage/types";
import { client } from "../../storage/client";
import { Paper } from "../../components/Paper";

export function CreateWorkspace() {
  const navigate = useNavigate();

  const onSubmit = async (v: Workspace) => {
    const { error } = await client.POST("/api/workspaces", {
      body: v,
    });

    if (!error) {
      notifications.show({
        title: "Success",
        message: "Workspace created",
        color: "green",
      });
      navigate(`/workspaces/${v.name}`);
    }
  };

  return (
    <>
      <PageHeader
        title="Add workspace"
        breadcrumb={[{ label: "Workspaces" }, { label: "Add" }]}
      />
      <Paper>
        <WorkspaceForm initialValues={{} as Workspace} onSubmit={onSubmit} />
      </Paper>
    </>
  );
}
