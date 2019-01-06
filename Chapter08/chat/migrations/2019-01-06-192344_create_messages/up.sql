CREATE TABLE messages (
  id SERIAL PRIMARY KEY,
  timestamp TIMESTAMP NOT NULL,
  channel_id INTEGER NOT NULL REFERENCES channels,
  user_id INTEGER NOT NULL REFERENCES users,
  text TEXT NOT NULL
);

SELECT diesel_manage_updated_at('messages');
