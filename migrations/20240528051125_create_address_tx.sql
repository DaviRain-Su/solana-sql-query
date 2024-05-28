-- Add migration script here
CREATE TABLE address_tx (
    id uuid NOT NULL,
    tx_signature TEXT NOT NULL UNIQUE,
    address TEXT NOT NULL,
    sequence_number BIGINT NOT NULL,  -- 添加递增的序列号

    PRIMARY KEY (id),
    UNIQUE (address, sequence_number)  -- 确保地址和序列号的组合是唯一的
);
