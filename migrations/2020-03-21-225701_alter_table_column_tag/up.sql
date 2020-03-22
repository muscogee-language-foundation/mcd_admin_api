ALTER TABLE entries
  RENAME COLUMN tag to tags;
ALTER TABLE entries
ADD
  COLUMN english_definitions VARCHAR;
