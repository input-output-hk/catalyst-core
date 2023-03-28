-- Cleanup if we already ran this before.
drop database if exists "CatalystEventDev";
drop user if exists "catalyst-event-dev";

-- Create the test user we will use with the local Catalyst-Event dev database.
create user "catalyst-event-dev" with password 'CHANGE_ME';

-- Privileges for this user/role.
alter default privileges revoke execute on functions from public;
alter default privileges in schema public revoke execute ON functions FROM "catalyst-event-dev";

-- Create the database.
create database "CatalystEventDev"
    with owner "catalyst-event-dev";

comment on database "CatalystEventDev" is 'Local Test Catalyst Event DB';
