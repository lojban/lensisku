-- Fix valsi type: gismu/cmavo only when there is a dictionary entry from user "officialdata".
-- Otherwise use experimental gismu (7) or experimental cmavo (8).
-- Type IDs: 1 = gismu, 2 = cmavo, 7 = experimental gismu, 8 = experimental cmavo.

-- Set gismu (1) -> experimental gismu (7) where no definition from officialdata exists for that valsi
UPDATE valsi v
SET typeid = 7
WHERE v.typeid = 1
  AND NOT EXISTS (
    SELECT 1
    FROM definitions d
    JOIN users u ON d.userid = u.userid
    WHERE d.valsiid = v.valsiid
      AND u.username = 'officialdata'
  );

-- Set cmavo (2) -> experimental cmavo (8) where no definition from officialdata exists for that valsi
UPDATE valsi v
SET typeid = 8
WHERE v.typeid = 2
  AND NOT EXISTS (
    SELECT 1
    FROM definitions d
    JOIN users u ON d.userid = u.userid
    WHERE d.valsiid = v.valsiid
      AND u.username = 'officialdata'
  );
