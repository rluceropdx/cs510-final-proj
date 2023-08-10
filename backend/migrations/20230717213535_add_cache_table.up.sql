CREATE TABLE IF NOT EXISTS cache
(
    id              serial PRIMARY KEY,
    api_type        VARCHAR(255) NOT NULL,
    url             VARCHAR(1000) NOT NULL,
    search_terms    VARCHAR(255) NOT NULL,
    json_results    TEXT,
    created_on TIMESTAMPTZ    NOT NULL DEFAULT NOW()
);
