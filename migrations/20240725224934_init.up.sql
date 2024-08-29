-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    "addresses" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        city VARCHAR(100) NOT NULL,
        state VARCHAR(100) NOT NULL,
        zip VARCHAR(100) NOT NULL,
        line_1 VARCHAR(255) NOT NULL,
        line_2 VARCHAR(255)
    );


CREATE TABLE
    "images" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        path VARCHAR(255) NOT NULL,
        alt VARCHAR(255) NOT NULL,
        subtitle TEXT
    );


CREATE TABLE
    "dedications" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        name VARCHAR(100) NOT NULL,
        bio TEXT NOT NULL,
        birth DATE NOT NULL,
        death DATE NOT NULL,
        img_ids UUID[] NOT NULL
    );


CREATE TABLE
    "patrol_logs" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        heading VARCHAR(255) NOT NULL,
        description TEXT NOT NULL,
        date DATE NOT NULL,
        img_ids UUID[] NOT NULL
    );


CREATE TABLE
    "testimonials" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        firstname VARCHAR(100) NOT NULL,
        lastname VARCHAR(100) NOT NULL,
        bio TEXT,
        content TEXT NOT NULL
    );

CREATE TABLE
    "support_resources" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        name VARCHAR(100) NOT NULL,
        description TEXT NOT NULL,
        missions TEXT[] NOT NULL,
        phone VARCHAR(100),
        email VARCHAR(255),
        address_id UUID,
        FOREIGN KEY (address_id) REFERENCES addresses(id)
    );

