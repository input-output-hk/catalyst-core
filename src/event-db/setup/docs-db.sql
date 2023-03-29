-- Cleanup if we already ran this before.
drop database if exists "CatalystEventDocs";
drop user if exists "catalyst-event-docs";

-- Create the test user we will use with the local Catalyst-Event dev database.
create user "catalyst-event-docs" with password 'CHANGE_ME';

-- Create the database.
create database "CatalystEventDocs"
    with owner "catalyst-event-docs";

comment on database "CatalystEventDocs" is 'Catalyst Event DB';
