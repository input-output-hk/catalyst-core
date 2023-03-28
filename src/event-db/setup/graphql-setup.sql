
\connect CatalystEventDev;

-- This is NOT part of our schema proper.  It exists to support GraphQL Admin authorization.

create schema private;
alter default privileges in schema private revoke execute ON functions FROM public;

create extension if not exists "pgcrypto";

-- Defined Roles
drop role if exists "cat_admin";
create role "cat_admin" login password 'CHANGE_ME';
comment on role "cat_admin" IS 'Full Administrator Access to the Database.';
GRANT "cat_admin" TO "catalyst-event-dev"; -- Make this role available to our default DB user.

alter default privileges in schema public revoke EXECUTE ON FUNCTIONS FROM cat_admin;
alter default privileges in schema private revoke EXECUTE ON FUNCTIONS FROM cat_admin;

drop role if exists "cat_anon";
create role "cat_anon" login password 'CHANGE_ME';
comment on role "cat_anon" IS 'Unauthenticated Read Only Access to the Database.';
GRANT "cat_anon" TO "catalyst-event-dev"; -- Make this role available to our default DB user.

alter default privileges in schema public revoke EXECUTE ON FUNCTIONS FROM cat_anon;
alter default privileges in schema private revoke EXECUTE ON FUNCTIONS FROM cat_anon;

-- Allow all roles "usage" to private and public schemas

grant usage on schema public  to "catalyst-event-dev", "cat_admin", "cat_anon";
grant usage on schema private to "catalyst-event-dev", "cat_admin", "cat_anon";

-- All roles can read the database
grant select on all tables in schema public to "catalyst-event-dev", "cat_anon", "cat_admin";

-- Only admin level roles can change data.
grant insert, update, delete on all tables in schema public to "catalyst-event-dev", "cat_admin";

-- Table where we store our authenticated users information.
-- No user has direct access to this table.
drop table if exists private.admin_account;
create table private.admin_account (
    id              serial primary key,
    first_name      text not null check (char_length(first_name) < 80),
    last_name       text check (char_length(last_name) < 80),
    role            text,
    about           text,
    email           text not null unique check (email ~* '^.+@.+\..+$'),
    password_hash   text not null,
    created_at      timestamp default now()
);

-- Function to register an admin.
-- Should ONLY be executable by admin level accounts.
drop function if exists private.register_admin;
create function private.register_admin(
  first_name text,
  last_name text,
  email text,
  password text,
  role text default 'cat_admin'
) returns private.admin_account as $$
declare
  admin private.admin_account;
begin
  insert into private.admin_account (first_name, last_name, role, email, password_hash) values
    (first_name, last_name, role, email, crypt(password, gen_salt('bf')))
    returning * into admin;

  admin.password_hash := ('xxx');

  return admin;
end;
$$ language plpgsql strict security definer;

comment on function private.register_admin(text, text, text, text, text) is 'Registers a single admin user and creates an account.';

grant execute on function private.register_admin(text, text, text, text, text) to "catalyst-event-dev", "cat_admin";

-- Create a default Admin User.  CHANGE_ME...
select private.register_admin('Default','Admin','nothing@nowhere.com','CHANGE_ME');

-- Define special type for JWT Authentication logic in server

drop type if exists private.jwt_token;
create type private.jwt_token as (
  role text,
  admin_id integer,
  exp bigint
);

-- Function to authenticate, GraphQL server uses this to generate the JWT.

drop function if exists private.authenticate;
create function private.authenticate(
  email text,
  password text
) returns private.jwt_token as $$
declare
  account private.admin_account;
begin
  select a.* into account
  from private.admin_account as a
  where a.email = $1;

  if account.password_hash = crypt(password, account.password_hash) then
    return (account.role, account.id, extract(epoch from (now() + interval '1:00:00')))::private.jwt_token;
  else
    return null;
  end if;
end;
$$ language plpgsql strict security definer;

comment on function private.authenticate(text, text) is 'Creates a JWT token that will securely identify a person and give them certain permissions. This token expires in 1 hours.';

grant execute on function private.authenticate(text, text) to "catalyst-event-dev", "cat_anon", "cat_admin";

-- Test it works with the default user.  CHANGE_ME...
select private.authenticate('nothing@nowhere.com','CHANGE_ME');

-- Function to get details about the current authenticated account.
-- Anyone can call this.
drop function if exists private.current_acct;
create function private.current_acct() returns private.admin_account as $$
declare
  account private.admin_account;
begin
  select a.* into account
  from private.admin_account as a
  where id = nullif(current_setting('jwt.claims.admin_id', true), '')::integer;

  if not FOUND then
    account.id := -1;
    account.first_name := ('');
    account.role := ('cat_anon');
    account.email := ('');
    account.created_at := now();
  end if;

  account.password_hash := ('xxx');

  return account;
end;
$$ language plpgsql stable security definer;

comment on function private.current_acct() is 'Gets the account identified by our JWT.';

grant execute on function private.current_acct() to "catalyst-event-dev", "cat_anon", "cat_admin";

-- Test if we can get the current account.
select private.current_acct();
