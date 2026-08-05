import { useForm } from "@mantine/form";

import type { Workspace } from "../../storage/types";
import { Textarea, TextInput, Button, Group } from "@mantine/core";

interface IProps {
  update?: boolean;
  initialValues: Workspace;
  onSubmit: (v: Workspace) => void;
}

export function WorkspaceForm(props: IProps) {
  const form = useForm({
    mode: "uncontrolled",
    initialValues: props.initialValues,
    validate: {
      name: (value) =>
        /^[a-z0-9\-]*$/.test(value) ? null : "Enter a valid name",
    },
  });

  return (
    <form onSubmit={form.onSubmit((v) => props.onSubmit(v))}>
      <TextInput
        withAsterisk
        label="Name"
        placeholder="workspace-name-123"
        key={form.key("name")}
        disabled={props.update}
        {...form.getInputProps("name")}
      />
      <Textarea
        mt="md"
        label="Description"
        key={form.key("description")}
        minRows={10}
        autosize
        {...form.getInputProps("description")}
      />
      <Group justify="flex-end" mt="md">
        <Button type="submit">Submit</Button>
      </Group>
    </form>
  );
}
