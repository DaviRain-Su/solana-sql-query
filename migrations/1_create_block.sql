CREATE TABLE ui_confirmed_blocks (
    id SERIAL PRIMARY KEY,
    previous_blockhash VARCHAR(255) NOT NULL,
    blockhash VARCHAR(255) NOT NULL,
    parent_slot BIGINT NOT NULL,
    block_time BIGINT,
    block_height BIGINT,
    transactions JSONB DEFAULT NULL,
    signatures JSONB DEFAULT NULL,
    rewards JSONB DEFAULT NULL
);
