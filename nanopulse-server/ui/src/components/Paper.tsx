import { Paper as PaperOriginal } from "@mantine/core";
import { ReactElement } from "react";

export function Paper(props: {
  children?: ReactElement | ReactElement[] | undefined;
}) {
  return (
    <PaperOriginal mt="md" style={{ padding: "var(--mantine-spacing-md)" }}>
      {props.children}
    </PaperOriginal>
  );
}
