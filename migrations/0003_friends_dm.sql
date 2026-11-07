-- Friendship is simplified: adding someone by username immediately makes
-- you friends with each other (no accept/decline step for now).
CREATE TABLE friends (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    friend_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, friend_id)
);

-- One conversation per unique pair of users. user_a is always the
-- lexicographically smaller UUID, so (A,B) and (B,A) never create two rows.
CREATE TABLE conversations (
    id UUID PRIMARY KEY,
    user_a UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_b UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (user_a, user_b)
);

CREATE TABLE dm_messages (
    id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);