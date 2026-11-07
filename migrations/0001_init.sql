CREATE TABLE servers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE channels (
    id UUID PRIMARY KEY,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name TEXT NOT NULL
);

CREATE TABLE messages (
    id UUID PRIMARY KEY,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    username TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed a default server/channel so the app has something to show on first run.
INSERT INTO servers (id, name) VALUES ('00000000-0000-0000-0000-000000000001', 'My Server');
INSERT INTO channels (id, server_id, name)
    VALUES ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 'general');
