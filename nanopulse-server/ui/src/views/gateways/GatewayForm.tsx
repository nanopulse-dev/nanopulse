import { useState, useEffect } from "react";
import { useForm, isNotEmpty, matches } from "@mantine/form";

import type { Gateway, RegionListItem } from "../../storage/types";
import { Textarea, TextInput, Button, Group, Select } from "@mantine/core";

import { client } from "../../storage/client";

interface IProps {
  initialValues: Gateway;
  update?: boolean;
  onSubmit: (v: Gateway) => void;
}

export function GatewayForm(props: IProps) {
  const [regions, setRegions] = useState<RegionListItem[]>([]);

  useEffect(() => {
    const fetch = async () => {
      const { data } = await client.GET("/api/regions");

      if (data) {
        setRegions(data.result);
      }
    };

    fetch();
  }, []);

  const form = useForm({
    mode: "uncontrolled",
    initialValues: props.initialValues,
    validate: {
      name: matches(/^[a-z0-9\-]+$/, "Enter a valid name"),
      region_module: isNotEmpty("Select a region configuration"),
    },
  });

  return (
    <form onSubmit={form.onSubmit((v) => props.onSubmit(v))}>
      <TextInput
        withAsterisk
        label="Name"
        placeholder="gateway-name-123"
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
      <Select
        withAsterisk
        mt="md"
        label="Region configuration"
        key={form.key("region_module")}
        data={regions.map((v) => ({
          value: v.module,
          label: v.name,
        }))}
        {...form.getInputProps("region_module")}
      />
      <Group justify="flex-end" mt="md">
        <Button type="submit">Submit</Button>
      </Group>
    </form>
  );
}
