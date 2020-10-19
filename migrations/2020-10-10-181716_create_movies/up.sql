-- Your SQL goes here
CREATE TABLE periods (
    id INTEGER NOT NULL PRIMARY KEY,
    start_day BIGINT NOT NULL,
    end_day BIGINT
);

CREATE TABLE submissions (
    id INTEGER NOT NULL PRIMARY KEY,
    dis_user_id VARCHAR(20) NOT NULL,
    title VARCHAR NOT NULL,
    link VARCHAR NOT NULL,
    period_id INTEGER NOT NULL,
    FOREIGN KEY(period_id) REFERENCES periods(id)
);

CREATE TABLE rolls (
    id INTEGER NOT NULL PRIMARY KEY,
    selection_1 INTEGER NOT NULL,
    selection_2 INTEGER NOT NULL,
    period_id INTEGER NOT NULL,
    FOREIGN KEY(selection_1) REFERENCES submissions(id),
    FOREIGN KEY(selection_2) REFERENCES submissions(id),
    FOREIGN KEY(period_id) REFERENCES periods(id)
);