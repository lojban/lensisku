-- Tag is the language code. Names stay free to repeat; only the code is unique.
CREATE UNIQUE INDEX languages_tag_lower_uidx ON languages (lower(tag));
