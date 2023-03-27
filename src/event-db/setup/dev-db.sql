-- Cleanup if we already ran this before.
drop database if exists "CatalystEventDev";
drop user if exists "catalyst-event-dev";

-- Create the test user we will use with the local Catalyst-Event dev database.
create user "catalyst-event-dev" with password 'CHANGE_ME';

-- Create the database.
create database "CatalystEventDev"
    with owner "catalyst-event-dev";

comment on database "CatalystEventDev" is 'Local Test Catalyst Event DB';
