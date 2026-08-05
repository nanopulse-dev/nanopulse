create table workspace (
  id integer primary key,
  name text not null,
  created_at datetime not null,
  updated_at datetime not null,
  description text not null
);

create unique index idx_workspace_name on workspace (name);

create table gateway (
  id integer primary key,
  workspace_id integer not null references workspace on delete cascade,
  name text not null,
  created_at datetime not null,
  updated_at datetime not null,
  last_heartbeat_at datetime null,
  description text not null,
  region_module text not null,
  allow_key_exchange_until datetime null
);

create unique index idx_gateway_name on gateway (name);

create table device (
  id integer primary key,
  workspace_id integer not null references workspace on delete cascade,
  name text not null,
  public_key blob not null,
  short_id blob not null,
  created_at timestamp not null,
  updated_at timestamp not null,
  key_exchange_at timestamp null,
  activation_at timestamp null,
  pin blob not null,
  description text not null,
  vendor_id blob not null default x'',
  profile_id blob not null default x'',
  version_id blob not null default x'',
  vendor_name text not null,
  telemetry text not null default '{}',
  state text not null default '{}',
  state_desired text not null default '{}',
  configuration text not null default '{}',
  configuration_desired text not null default '{}',
  mac_configuration text not null default '{}',
  mac_configuration_desired not null default '{}',
  root_key blob not null default x'',
  session_root_key blob not null default x'',
  counters text not null default '{}'
);

create index idx_device_short_id on device (short_id);
create unique index idx_device_name on device (name);
create unique index idx_device_public_key on device (public_key);

create table device_key_exchange (
  id integer primary key,
  device_id integer not null references device on delete cascade,
  created_at datetime not null,
  request_nonce integer not null,
  response_nonce integer not null,
  root_key blob not null
);

create index idx_device_key_exchange_device_id on device_key_exchange (device_id);
