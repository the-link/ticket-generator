-- Add migration script here
CREATE TABLE IF NOT EXISTS issues_reported
(
    uuid          UUID PRIMARY KEY,
    issue_name      VARCHAR NOT NULL,
    description      VARCHAR NOT NULL,
    reported_by      VARCHAR NOT NULL,
    company_name      VARCHAR NOT NULL,
    contact_number      bigint NOT NULL,
    ticket_number      bigint NOT NULL,
    ticket_owner      VARCHAR NOT NULL,
    status      VARCHAR NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
