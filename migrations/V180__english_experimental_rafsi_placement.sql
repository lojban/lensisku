-- No assignment changes for mabla, zabna, gleki, kanpe or carna.
-- Reviewed destinations for explicit experimental rafsi proposals.
-- Automatic four-letter experimental gismu stems are not authorship proposals;
-- they keep their existing official/experimental morphology distinction.
-- Note-only saf/sif/suf are preserved in V179 archive copies, not silently activated.
-- A reserved non-login archive identity, not a claim of original authorship.
INSERT INTO users(username,email,password,created_at,role,email_confirmed,votesize,realname,personal)
SELECT 'nalcatni stidi','nalcatni-stidi@archive.invalid','DISABLED',now(),'blocked',false,0,
       'nalcatni stidi','Preserved unofficial definitions and rafsi proposals. See each record history for its source; this account is not their original proposer.'
WHERE NOT EXISTS (SELECT 1 FROM users WHERE username='nalcatni stidi');
DO $$ BEGIN
 IF NOT EXISTS (SELECT 1 FROM users WHERE username='nalcatni stidi'
                AND email='nalcatni-stidi@archive.invalid' AND password='DISABLED' AND role='blocked') THEN
  RAISE EXCEPTION 'Archive username already belongs to a different account';
 END IF;
 IF NOT EXISTS (SELECT 1 FROM languages WHERE langid=2 AND tag='en') THEN
  RAISE EXCEPTION 'Expected English language id 2';
 END IF;
END $$;
LOCK TABLE definitions IN SHARE ROW EXCLUSIVE MODE;
-- Helpers are session-local; no permanent authority table or status flag.
CREATE OR REPLACE FUNCTION pg_temp.record_repair_version(p_id INTEGER, p_actor INTEGER, p_message TEXT)
RETURNS VOID LANGUAGE SQL AS $$
 INSERT INTO definition_versions
 (definition_id, langid, valsiid, definition, notes, etymology, selmaho, jargon, rafsi,
  gloss_keywords, place_keywords, user_id, message, word, owner_only, created_at)
 SELECT d.definitionid, d.langid, d.valsiid, d.definition, d.notes, d.etymology,
        d.selmaho, d.jargon, editable_definition_rafsi(d.definitionid),
        COALESCE((SELECT jsonb_agg(jsonb_build_object('word',n.word,'meaning',n.meaning) ORDER BY n.word)
                  FROM keywordmapping k JOIN natlangwords n ON n.wordid=k.natlangwordid
                  WHERE k.definitionid=d.definitionid AND k.place=0), '[]'::jsonb),
        COALESCE((SELECT jsonb_agg(jsonb_build_object('word',n.word,'meaning',n.meaning) ORDER BY k.place,n.word)
                  FROM keywordmapping k JOIN natlangwords n ON n.wordid=k.natlangwordid
                  WHERE k.definitionid=d.definitionid AND k.place>0), '[]'::jsonb),
        p_actor, p_message, v.word, d.owner_only, clock_timestamp()
 FROM definitions d JOIN valsi v ON v.valsiid=d.valsiid WHERE d.definitionid=p_id;
$$;
CREATE OR REPLACE FUNCTION pg_temp.archive_definition(p_source INTEGER, p_actor INTEGER, p_reason TEXT)
RETURNS INTEGER LANGUAGE plpgsql AS $$
DECLARE original definitions%ROWTYPE; copy_id INTEGER; original_author TEXT;
BEGIN
 SELECT * INTO STRICT original FROM definitions WHERE definitionid=p_source;
 SELECT username INTO STRICT original_author FROM users WHERE userid=original.userid;
 SELECT definitionid INTO copy_id FROM definitions
 WHERE userid=p_actor AND langid=2 AND valsiid=original.valsiid
   AND metadata->'nalcatni_stidi_archive'->>'source_definition_id'=p_source::text;
 IF FOUND THEN RETURN copy_id; END IF;
 INSERT INTO definitions
 (definitionid, langid, valsiid, definitionnum, definition, notes, etymology, selmaho,
  jargon, userid, time, owner_only, metadata, rafsi)
 VALUES (nextval('definitions_definitionid_seq'),2,original.valsiid,
         (SELECT COALESCE(max(definitionnum),0)+1 FROM definitions WHERE valsiid=original.valsiid AND langid=2),
         original.definition,original.notes,original.etymology,original.selmaho,original.jargon,
         p_actor,extract(epoch FROM now())::integer,false,
         original.metadata || jsonb_build_object('nalcatni_stidi_archive',jsonb_build_object(
           'source_definition_id',p_source,'source_author',original_author,
           'source_language_id',original.langid,'source_time',original.time,'reason',p_reason)),
         original.rafsi)
 RETURNING definitionid INTO copy_id;
 INSERT INTO keywordmapping(definitionid,natlangwordid,place)
 SELECT copy_id,natlangwordid,place FROM keywordmapping WHERE definitionid=p_source;
 INSERT INTO definition_images(definition_id,image_data,mime_type,description,display_order,created_by)
 SELECT copy_id,image_data,mime_type,description,display_order,created_by
 FROM definition_images WHERE definition_id=p_source;
 PERFORM pg_temp.record_repair_version(copy_id,p_actor,'Preserved source definition #'||p_source||': '||p_reason);
 RETURN copy_id;
END;
$$;
CREATE TEMP TABLE rafsi_english_plan(word TEXT,token TEXT,target_definition_id INTEGER,
 archive_target BOOLEAN,allow_non_english BOOLEAN,expected_author TEXT,reason TEXT) ON COMMIT DROP;
INSERT INTO rafsi_english_plan VALUES
('bekpi','bek',36701,false,false,'gleki','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('bolva','bov',88625,false,false,'loblat','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor; proposer MoheXoheKohe documented in comment #5358 has no current definition of this word'),
('brode','bo''e',2527,false,false,'noralujv','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('cketi','cet',75027,false,false,'loblat','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('cnasi','nas',75948,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('co''i','co''i',7741,false,true,'xorxes','unknown proposer; no English nonofficial definition, use earliest nonofficial definition; Japanese quotation does not prove authorship'),
('corci','coc',56846,false,false,'krtisfranks','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('cpixa','cix',72255,false,false,'zozeizeizeizeifaho','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('cumba','cub',72150,false,false,'zozeizeizeizeifaho','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('dau','duv',1467,true,false,'officialdata','explicit imported proposal notes; wiki credits rab.spir, who has no matching current definition of this word'),
('ditcu','dit',15153,false,false,'xorxes','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor; proposer selpahi documented in comment #648 has no current definition of this word'),
('dutso','tso',66167,false,false,'krtisfranks','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('dzodu','dzo',75333,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('farje','faj',75306,false,false,'mati!','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('fei','fel',1541,true,false,'officialdata','explicit imported proposal notes; wiki credits rab.spir, who has no matching current definition of this word'),
('fibra','fib',70337,false,false,'k1234567890y','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('flese','les',56773,false,false,'krtisfranks','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('flipo','lip',69606,false,false,'viktor','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('fokma','fok',75714,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('folcu','fol',88665,false,false,'mati!','English notes explicitly specify combining form -fol-'),
('gai','gam',1572,true,false,'officialdata','explicit imported proposal notes; wiki credits rab.spir, who has no matching current definition of this word'),
('gajno','gaj',75812,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('ganvi','gav',69196,false,false,'krtisfranks','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('gelga','geg',73769,false,false,'Qimar','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor; proposer loblat documented in comment #5337 has no current definition of this word'),
('gomsi','gos',57072,false,false,'krtisfranks','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('grepu','gep',75314,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('jau','juz',1652,true,false,'officialdata','explicit imported proposal notes; wiki credits rab.spir, who has no matching current definition of this word'),
('jguna','jgu',73599,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('jguza','zvo',75621,false,true,'gleki','gleki history #1947 records zvo; no English definition by this user exists, retain their earliest nonofficial test-language definition'),
('jicfo','cfo',72225,false,false,'zozeizeizeizeifaho','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('jidge','jid',72688,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('jonse','jos',67049,false,false,'spheniscine','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('kazgo','zgo',75346,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('kenjo','kej',76120,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('kibro','kib',16345,false,false,'daniel','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('kirgo','kig',75344,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('korvo','kov',15634,false,false,'phma','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('kotfo','kof',75156,false,false,'ezras','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('kroxo','kox',69493,false,false,'melop','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('lokra','lok',60531,false,false,'phma','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('losxa','lo''a',75258,false,false,'mati!','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('losxa','lo''a',75255,false,false,'mati!','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('losxa','los',75258,false,false,'mati!','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('losxa','los',75255,false,false,'mati!','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('ma','maz',88840,false,false,'MoheXoheKohe','unknown proposer; earliest existing English nonofficial definition, otherwise earliest existing nonofficial definition; no claim of authorship'),
('majgo','jgo',74322,false,false,'mati','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('mo','moz',7101,false,true,'xorxes','unknown proposer; earliest existing English nonofficial definition, otherwise earliest existing nonofficial definition; no claim of authorship'),
('mongo','mog',76202,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('moxna','mox',74167,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('moxna','mox',69347,false,false,'krtisfranks','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('nedlo','ned',73229,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('neplo','nep',75842,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('nijro','nij',75343,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('norgo','nog',16923,false,false,'lindarthebard','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('nudle','nud',52900,false,false,'glekizmiku','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('pekni','pek',75130,false,false,'mati','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('ploxa','lox',88311,false,false,'mati!','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('podji','pod',75413,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('pombo','pom',73600,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('posko','pok',74313,false,false,'mati','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('purba','pub',75317,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('sefmi','sem',75572,false,false,'mati!','explicit proposal by mati! in comment #5091; prefer their definition'),
('sekse','sek',75106,false,false,'mati','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('sfeno','se''o',74245,false,false,'mati','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('sfite','sfi',72679,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('so''y','sox',88898,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('tceta','cet',73249,false,false,'Suskeyhose','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('tonsi','tos',73587,false,false,'lalxu','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('tuvju','tuv',75365,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('vai','vav',2321,true,false,'officialdata','explicit imported proposal notes; wiki credits rab.spir, who has no matching current definition of this word'),
('vedli','ve''i',38159,false,false,'gleki','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('vente','vet',65910,false,false,'gleki','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('venzi','vez',75100,false,false,'mati','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('vetno','ve''o',88652,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('vo''a','vob',2353,true,false,'officialdata','explicit imported proposal notes'),
('vujnu','vu''u',66912,false,false,'spheniscine','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('vujnu','vuj',66912,false,false,'spheniscine','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('xei','xem',16190,false,false,'spheniscine','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor; wiki credits rab.spir, who has no matching current definition of this word'),
('xelfo','xef',76093,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('xo','xlo',88839,false,false,'MoheXoheKohe','unknown proposer; earliest existing English nonofficial definition, otherwise earliest existing nonofficial definition; no claim of authorship'),
('xrotu','xro',65733,false,false,'selckiku','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('xundo','xud',76279,false,false,'MoheXoheKohe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('za''u','zab',88594,false,false,'bairyn','explicit proposal by bairyn in comment #5355; prefer their definition'),
('zandi','zad',72717,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('zanfu','zaf',75729,false,false,'mati!','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('zobri','zob',75632,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('zobri','zoi',75632,false,false,'janbe','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('zucna','zu''a',66166,false,false,'krtisfranks','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('zucna','zuc',66166,false,false,'krtisfranks','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('zviki','zvi',67311,false,false,'spheniscine','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor'),
('zvomo','zvo',74251,false,false,'mati','retain existing English proposal record(s); do not infer original proposer from a definition author or later editor');

DO $$
DECLARE item RECORD; actor INTEGER; destination INTEGER; source RECORD; new_rafsi TEXT;
BEGIN
 SELECT userid INTO STRICT actor FROM users WHERE username='nalcatni stidi';
 -- Validate the entire manifest before mutating assignments. IDs must identify
 -- the expected Lojban word; never turn a stale ID into an unrelated assignment.
 IF EXISTS(SELECT 1 FROM rafsi_english_plan p
           WHERE p.target_definition_id IS NOT NULL
             AND EXISTS(SELECT 1 FROM valsi v WHERE v.word=p.word AND v.source_langid=1)
             AND NOT EXISTS(SELECT 1 FROM definitions d JOIN valsi v ON v.valsiid=d.valsiid JOIN users u ON u.userid=d.userid
                            WHERE d.definitionid=p.target_definition_id AND v.word=p.word AND v.source_langid=1
                              AND u.username=p.expected_author
                              AND (p.archive_target OR u.username<>'officialdata')
                              AND (p.archive_target OR p.allow_non_english OR d.langid=2))) THEN
  RAISE EXCEPTION 'English rafsi placement manifest no longer matches definition identities';
 END IF;
 FOR item IN SELECT * FROM rafsi_english_plan ORDER BY word,token,target_definition_id LOOP
  IF NOT EXISTS(SELECT 1 FROM valsi WHERE word=item.word AND source_langid=1) THEN CONTINUE; END IF;
  IF item.target_definition_id IS NOT NULL THEN
   IF item.archive_target THEN
    destination:=pg_temp.archive_definition(item.target_definition_id,actor,item.reason);
   ELSE destination:=item.target_definition_id; END IF;
   SELECT combined_rafsi(d.rafsi,item.token) INTO new_rafsi FROM definitions d WHERE d.definitionid=destination;
   IF (SELECT rafsi FROM definitions WHERE definitionid=destination) IS DISTINCT FROM new_rafsi THEN
    PERFORM pg_temp.record_repair_version(destination,actor,'Before English rafsi placement: '||item.reason);
    UPDATE definitions SET rafsi=new_rafsi,time=extract(epoch FROM now())::integer WHERE definitionid=destination;
    PERFORM pg_temp.record_repair_version(destination,actor,'English experimental rafsi -'||item.token||'-: '||item.reason);
   END IF;
  END IF;
 END LOOP;
 -- Remove a token only after all its English destinations exist. Keep multiple
 -- English interpretations: ownership of a token resolves by word, not version.
 FOR source IN
  SELECT d.definitionid,d.rafsi,p.word
  FROM definitions d JOIN valsi v ON v.valsiid=d.valsiid
  JOIN (SELECT DISTINCT word FROM rafsi_english_plan) p ON p.word=v.word
  WHERE v.source_langid=1
 LOOP
  SELECT string_agg(token,' ' ORDER BY token) INTO new_rafsi
  FROM (SELECT DISTINCT rafsi_tokens.token
        FROM regexp_split_to_table(COALESCE(source.rafsi,''),'\s+') AS rafsi_tokens(token)
        WHERE rafsi_tokens.token<>'' AND NOT EXISTS(
          SELECT 1 FROM rafsi_english_plan p WHERE p.word=source.word AND p.token=rafsi_tokens.token
           AND NOT EXISTS(SELECT 1 FROM rafsi_english_plan keep
                          WHERE keep.word=p.word AND keep.token=p.token
                           AND CASE WHEN keep.archive_target THEN
                              (SELECT metadata->'nalcatni_stidi_archive'->>'source_definition_id'
                               FROM definitions WHERE definitionid=source.definitionid)=keep.target_definition_id::text
                              AND (SELECT u.username FROM definitions d JOIN users u ON u.userid=d.userid
                                   WHERE d.definitionid=source.definitionid)='nalcatni stidi'
                            ELSE keep.target_definition_id=source.definitionid END)
        )) retained;
  IF source.rafsi IS DISTINCT FROM new_rafsi THEN
   PERFORM pg_temp.record_repair_version(source.definitionid,actor,'Pre-placement snapshot; contains migration/inherited rafsi, not evidence of original authorship');
   UPDATE definitions SET rafsi=new_rafsi,time=extract(epoch FROM now())::integer WHERE definitionid=source.definitionid;
   PERFORM pg_temp.record_repair_version(source.definitionid,actor,'Moved explicit experimental rafsi to reviewed English definitions; implicit gismu stems retained');
  END IF;
 END LOOP;
END $$;
DELETE FROM cached_dictionary_exports;
