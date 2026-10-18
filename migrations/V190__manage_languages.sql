-- Editors of the dictionary-language catalog (add and edit). Viewing stays public.
-- trg_sync_admin_after_permission_changes grants this to the admin role.
INSERT INTO permissions (name, description)
VALUES (
    'manage_languages',
    'Can add and edit languages used for corpus entries and translations'
);
