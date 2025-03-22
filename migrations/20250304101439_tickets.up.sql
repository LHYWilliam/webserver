-- Add up migration script here

create table tickets (
    id integer unique primary key autoincrement,
    title text not null
);
