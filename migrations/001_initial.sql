CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE roles (
    id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE locations (
    id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name    TEXT NOT NULL,
    address TEXT NOT NULL
);

CREATE TABLE categories (
    id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username      TEXT NOT NULL UNIQUE,
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role_id       UUID NOT NULL REFERENCES roles(id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE events (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    date        TIMESTAMPTZ NOT NULL,
    location_id UUID NOT NULL REFERENCES locations(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tabela łącząca (relacja N:N — jedno wydarzenie, wiele kategorii)
CREATE TABLE event_categories (
    event_id    UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (event_id, category_id)
);

CREATE TABLE reservations (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_id   UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, event_id)  -- jeden user = jedna rezerwacja na wydarzenie
);

-- Dane startowe
INSERT INTO roles (name) VALUES ('admin'), ('user');

INSERT INTO locations (name, address) VALUES
    ('Klub Sigma',        'ul. Nowy Świat 15, Warszawa'),
    ('Hala Expo',         'ul. Przemysłowa 7, Warszawa');

INSERT INTO categories (name) VALUES
    ('Muzyka'), ('Sport'), ('Kultura');

INSERT INTO events (title, description, date, location_id) VALUES
    ('Noc Jazzu',
     'Wieczór z najlepszym jazzem na żywo.',
     NOW() + INTERVAL '7 days',
     (SELECT id FROM locations WHERE name = 'Klub Sigma')),
    ('Turniej Tenisa',
     'Amatorski turniej tenisowy.',
     NOW() + INTERVAL '14 days',
     (SELECT id FROM locations WHERE name = 'Hala Expo'));