-- Create or replace view for convenient comment access to include definition_link_id
CREATE OR REPLACE VIEW convenientcomments AS
SELECT 
    c.commentid,
    c.threadid,
    c.parentid,
    c.userid,
    u.username,
    u.realname,
    c.time,
    c.subject,
    c.content,
    c.commentnum,
    cc.total_reactions,
    cc.total_replies,
    t.valsiid,
    t.definitionid,
    t.definition_link_id
FROM 
    comments c
    JOIN users u ON c.userid = u.userid
    JOIN threads t ON c.threadid = t.threadid
    LEFT JOIN comment_counters cc ON c.commentid = cc.comment_id;
