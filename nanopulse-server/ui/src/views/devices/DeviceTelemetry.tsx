import type { ReactElement } from "react";
import { Card, Center, Text, Grid } from "@mantine/core";
import { GaugeChart } from "@mantine/charts";
import Map, {
  Source,
  Layer,
  NavigationControl,
  FullscreenControl,
} from "react-map-gl/maplibre";
import circle from "@turf/circle";

import type { Profile, TemperatureProperties } from "../../storage/types";

interface Location {
  latitude: number;
  longitude: number;
  accuracy: number;
}

interface IProps {
  profile: Profile;
  telemetry?: Record<string, any>;
}

export function DeviceTelemetry(props: IProps) {
  let telemetry = Object.entries(props.profile.telemetry_schema).map(
    ([k, v]) => {
      let content: ReactElement = <></>;

      if ("sensor" in v.component && typeof v.component.sensor === "object") {
        if ("temperature" in v.component.sensor) {
          const temperature = props.telemetry?.[k] as number | undefined;
          content = (
            <TemperatureSensor
              properties={v.component.sensor.temperature}
              value={temperature}
            />
          );
        }
      } else if ("tracker" in v.component) {
        const location = props.telemetry?.[k] as Location | undefined;
        content = <TrackerMap value={location} />;
      } else if ("switch" in v.component) {
      }

      return (
        <Grid.Col span={{ base: 12, md: 6, lg: 4 }} key={k}>
          <Card withBorder padding={0}>
            <Card.Section py="xs" px="md">
              <Text fw={500}>{v.name}</Text>
            </Card.Section>
            {content}
          </Card>
        </Grid.Col>
      );
    },
  );

  return telemetry;
}

function TemperatureSensor(props: {
  properties: TemperatureProperties;
  value?: number;
}) {
  if (props.value) {
    return (
      <Center>
        <GaugeChart
          value={props.value}
          min={props.properties.min}
          max={props.properties.max}
          label={
            <Text ta="center" fz="xl">
              {props.value} {props.properties.unit}
            </Text>
          }
        />
      </Center>
    );
  } else {
    return <Card.Section>No data.</Card.Section>;
  }
}

function TrackerMap(props: { value?: Location }) {
  if (props.value) {
    const circleData = circle(
      [props.value.longitude, props.value.latitude],
      props.value.accuracy / 1000, // Turf expects kilometers
      { steps: 64, units: "kilometers" },
    );

    return (
      <Card.Section>
        <Map
          initialViewState={{
            latitude: props.value.latitude,
            longitude: props.value.longitude,
            zoom: 15,
          }}
          style={{ width: "100%", height: 350 }}
          mapStyle="https://tiles.openfreemap.org/styles/positron"
        >
          <NavigationControl />
          <FullscreenControl />
          <Source id="accuracy-circle" type="geojson" data={circleData}>
            <Layer
              id="accuracy-fill"
              type="fill"
              paint={{
                "fill-color": "#3388ff",
                "fill-opacity": 0.2,
              }}
            />
            <Layer
              id="accuracy-outline"
              type="line"
              paint={{
                "line-color": "#3388ff",
                "line-width": 2,
              }}
            />
          </Source>
        </Map>
      </Card.Section>
    );
  } else {
    return <Card.Section>No data.</Card.Section>;
  }
}
