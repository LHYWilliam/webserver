-- Add up migration script here

create table users (
    username text unique primary key not null,
    password text not null
);
