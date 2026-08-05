import { useForm } from "@mantine/form";

import type { Device } from "../../storage/types";
import { Textarea, TextInput, Button, Group } from "@mantine/core";

interface IProps {
  initialValues: Device;
  update?: boolean;
  onSubmit: (v: Device) => void;
}

export function DeviceForm(props: IProps) {
  const form = useForm({
    mode: "uncontrolled",
    initialValues: props.initialValues,
    validate: {
      pin: (value) => (/^[a-fA-F0-9]{8}$/.test(value) ? null : "Invalid PIN"),
    },
  });

  return (
    <form onSubmit={form.onSubmit((v) => props.onSubmit(v))}>
      <TextInput
        withAsterisk
        disabled={props.update}
        label="Public Key"
        key={form.key("public_key")}
        {...form.getInputProps("public_key")}
      />
      <TextInput
        mt="md"
        withAsterisk
        disabled={props.update}
        label="Short ID"
        key={form.key("short_id")}
        {...form.getInputProps("short_id")}
      />
      <TextInput
        mt="md"
        withAsterisk
        label="PIN"
        key={form.key("pin")}
        {...form.getInputProps("pin")}
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
