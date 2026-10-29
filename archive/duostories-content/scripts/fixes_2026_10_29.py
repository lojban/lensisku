"""Hand-reviewed corrections for dialogues.json (review of 2026-10-29).

Applied by scripts/apply_fixes.py.  Keys per story id:
  title   {'en','jbo'}           replace title
  lines   [(speaker, jbo, en)]   replace all lines (legacy conversions)
  L       {i: jbo | (jbo, en)}   replace line i
  Q       {i: question}          replace question i (None = delete)
  D       [(where, candidate, note)]  final doubtful list ('line N' / 'q N' / 'title')
  speakers [{'name','gender'}]
Global word / name substitutions are in SUBS and run first.
"""

# (regex, replacement) applied to every Lojban string
SUBS = [
    (r"\.bea\.", ".beas."),            # vowel-final cmevla
    (r"\.svetas\.", ".sfetas."),       # sv mixes voiced/unvoiced
    (r"\.maiakovskis\.", ".maiakofskis."),  # vsk mixes voicing
    (r"\.aleksei\.", ".alekseis."),    # vowel-final cmevla
    (r"\.mikelas\.", ".mikelys."),     # contains "la" (C19)
    (r"\bsanmycpa\b", "djasnuji"),     # sandwich (lujvo.tsv)
    (r"\bjdikargu\b", "kargu"),        # expensive = kargu (gismu)
    (r"\bpagbu'o\b", "jaspu"),         # passport (gismu)
    (r"\bbalcta'i\b", "balcu'e"),      # university (lujvo.tsv)
    (r"\bkruasona\b", "lurnanba"),     # croissant (lujvo.tsv)
    (r"\bpitsa\b", "pitnanba"),        # pizza (lujvo.tsv)
    (r"\btolcnino\b", "slabu"),        # old (of things): slabu (gismu); laldo is experimental
    (r"\blaldo\b", "slabu"),
    (r"\btamca stasu\b", "tamcystasu"),
    (r"\bvinji tcana\b", "vijytcana"),
]

FIXES = {}

FIXES["01-coi-ca-lo-cerni"] = dict(
    title={"en": "Good Morning", "jbo": "cerni coi"},
    speakers=[{"name": "Djan", "gender": "M"}, {"name": "Marian", "gender": "F"}],
    lines=[
        (">", ".i la .djan. .e la .marian. cu zvati lo zdani", "John and Marian are at home."),
        ("Djan", "cerni coi .marian.", "Good morning, Marian!"),
        ("Marian", "coi .djan.", "Hi, John."),
        ("Marian", ".i lo mi ckiku cu zvati ma", "Where is my key?"),
        ("Djan", ".i lo do ckiku cu cpana lo jubme", "Your key is on the table."),
        ("Marian", ".u'u .i mi tatpi", "Sorry, I'm tired."),
        ("Djan", ".i xu do djica lo nu pinxe lo ckafi", "Do you want (to drink) coffee?"),
        ("Marian", ".ie ki'e", "Yes, thanks."),
    ],
    questions=[
        {"type": "fill", "line_index": 3, "speaker": "Marian", "answer": "ckiku", "jbo": ".i lo mi [ckiku] cu zvati ma", "en": "Where is my key?"},
        {"type": "choice", "en": "Where is the key?", "jbo": ".i lo ckiku cu zvati ma", "answers": [
            {"en": "On the table.", "jbo": "lo jubme", "correct": True},
            {"en": "In the car.", "jbo": "lo karce", "correct": False},
            {"en": "In the coffee.", "jbo": "lo ckafi", "correct": False}]},
        {"type": "bank", "line_index": 6, "speaker": "Djan", "answer": "ckafi", "chips": ["jubme", "ckiku"], "jbo": ".i xu do djica lo nu pinxe lo [ckafi]", "en": "Do you want (to drink) coffee?"},
    ],
    D=[],
)

FIXES["02-sakta-joi-silna"] = dict(
    title={"en": "Sugar and Salt", "jbo": "lo sakta joi lo silna"},
    speakers=[{"name": "Marian", "gender": "F"}, {"name": "Djan", "gender": "M"}],
    lines=[
        (">", ".i la .marian. cu pinxe lo ckafi", "Marian is drinking coffee."),
        ("Marian", ".u'e", "Oh!"),
        ("Djan", ".i ma nabmi", "What's wrong?"),
        ("Marian", ".i ti na sakta .i ti silna", "This isn't sugar, it's salt!"),
        ("Djan", ".ua .i do mutce tatpi", "Wow, you're very tired."),
    ],
    questions=[
        {"type": "fill", "line_index": 3, "speaker": "Marian", "answer": "silna", "jbo": ".i ti na sakta .i ti [silna]", "en": "This isn't sugar, it's salt!"},
        {"type": "choice", "en": "What did Marian put in her coffee?", "jbo": ".i la .marian. cu punji ma lo ckafi", "answers": [
            {"en": "Salt.", "jbo": "lo silna", "correct": True},
            {"en": "Sugar.", "jbo": "lo sakta", "correct": False},
            {"en": "A key.", "jbo": "lo ckiku", "correct": False}]},
    ],
    D=[],
)

FIXES["03-lo-gusta"] = dict(
    title={"en": "The Restaurant", "jbo": "lo gusta"},
    speakers=[{"name": "Bea", "gender": "F"}, {"name": "Pedro", "gender": "M"}],
    lines=[
        (">", ".i la .beas. cu zvati lo gusta", "Bea is at a restaurant."),
        ("Bea", "coi .i xu do kanro", "Hi, are you well?"),
        ("Pedro", ".i mi kanro", "I'm well."),
        ("Pedro", ".i do djica lo nu citka ma", "What do you want to eat?"),
        ("Bea", "lo salta", "A salad."),
        ("Pedro", ".i mi djica lo nu citka lo nanba", "I want to eat bread."),
    ],
    questions=[
        {"type": "fill", "line_index": 4, "speaker": "Bea", "answer": "salta", "jbo": "lo [salta]", "en": "A salad."},
        {"type": "choice", "en": "What does Bea want to eat?", "jbo": ".i la .beas. cu djica lo nu citka ma", "answers": [
            {"en": "A salad.", "jbo": "lo salta", "correct": True},
            {"en": "Bread.", "jbo": "lo nanba", "correct": False},
            {"en": "Coffee.", "jbo": "lo ckafi", "correct": False}]},
    ],
    D=[("line 1", ".i xu do kanro", "kanro = healthy; as a greeting \"how are you\" it is a calque, kept because the English gloss is literally \"are you well?\"")],
)

FIXES["04-lo-taxfu"] = dict(
    title={"en": "The Red Coat", "jbo": "lo xunre kosta"},
    speakers=[],
    lines=[
        ("F", "coi .i mi djica lo nu te vecnu le vi xunre kosta", "Hi, I want to buy this red coat."),
        ("M", ".i lo kosta cu se jdima li mu", "The coat costs five."),
        ("F", ".ui .i mi te vecnu ri", "Great, I'll buy it."),
        ("M", "ki'e", "Thanks."),
        ("F", "je'e", "You're welcome."),
    ],
    questions=[
        {"type": "fill", "line_index": 1, "speaker": "M", "answer": "jdima", "jbo": ".i lo kosta cu se [jdima] li mu", "en": "The coat costs five."},
        {"type": "bank", "line_index": 0, "speaker": "F", "answer": "xunre", "chips": ["blanu", "crino"], "jbo": "coi .i mi djica lo nu te vecnu le vi [xunre] kosta", "en": "Hi, I want to buy this red coat."},
    ],
    D=[("line 4", "je'e", "je'e (\"roger\") as the answer to ki'e follows common usage; CLL gives no dedicated \"you're welcome\"")],
)

FIXES["05-lo-zarci"] = dict(
    title={"en": "Where Is the Shop?", "jbo": "lo zarci cu zvati ma"},
    speakers=[],
    lines=[
        ("M", "coi .i lo zarci cu zvati ma", "Hi, where is the shop?"),
        ("F", ".i lo zarci cu jibni lo ckule", "The shop is near the school."),
        ("M", ".i xu lo zarci cu darno", "Is the shop far?"),
        ("F", "na go'i .i ri jibni", "No, it's near."),
        ("M", "ki'e", "Thanks."),
    ],
    questions=[
        {"type": "fill", "line_index": 1, "speaker": "F", "answer": "jibni", "jbo": ".i lo zarci cu [jibni] lo ckule", "en": "The shop is near the school."},
        {"type": "choice", "en": "Is the shop far?", "jbo": ".i xu lo zarci cu darno", "answers": [
            {"en": "No, it's near.", "jbo": "na go'i .i ri jibni", "correct": True},
            {"en": "Yes, it's far.", "jbo": "go'i .i ri darno", "correct": False}]},
    ],
    D=[],
)

FIXES["batch2-01"] = dict(
    L={
        2: ".i .e'o mi djica lo nu do bevri lo kabri be lo djacu be'o .e lo salta",
        3: ".i'e .i lo kabri be lo djacu be'o .e lo patlu salta",
        4: "na go'i .i mi na djica lo nu citka lo patlu salta",
        13: (".i lo gusta cu se cmene lu lo patlu li'u", None),
    },
    Q={2: {"type": "choice", "en": "What happened in the story?", "jbo": ".i ma fasnu", "answers": [
        {"en": "Alina ate all the potato dishes.", "jbo": "la .alinas. cu citka ro lo patlu cidja", "correct": False},
        {"en": "Alina went to a restaurant that only served potato dishes.", "jbo": "la .alinas. cu klama lo gusta poi sabji lo patlu cidja po'o", "correct": True},
        {"en": "The waitress made Alina leave.", "jbo": "lo bevri cu rinka lo nu la .alinas. cu cliva lo gusta", "correct": False}]}},
    D=[],
)

FIXES["batch2-02"] = dict(
    L={
        7: ".ie .i so'i savru cu zvati lo tcadu",
        8: (".i ta sance ma", "What's that noise from?"),
        12: ".i mi ponse su'o djasnuji vi lo jupku'a",
        14: ".i .e'o mi djica lo nu lo karce cu bevri mi lo tcadu",
    },
    Q={3: {"type": "choice", "en": "What does Linda mean?", "jbo": ".i la .lindas. cu mo", "answers": [
        {"en": "She wants to leave the farm and go back to the city.", "jbo": "djica lo nu cliva lo cange gi'e klama lo tcadu", "correct": True},
        {"en": "She wants to move to the countryside.", "jbo": "djica lo nu xabju lo nurma", "correct": False},
        {"en": "She would like some sandwiches.", "jbo": "djica lo nu citka lo djasnuji", "correct": False}]}},
    D=[],
)

FIXES["batch2-03"] = dict(
    title={"en": "The Soccer Shoes", "jbo": "lo boltipyjvi cutci"},
    L={
        0: ".i la .karlos. .e la .lilianas. cu boltipyjvi",
        1: "doi .lilianas. .i mi'a boltipyjvi ca ro lo djedi",
        5: ".i xu ri kargu",
        7: ".i xu do ca certu lo ka boltipyjvi",
        8: (".i sepi'o le ti cutci mi mutce certu lo ka kelci", "With these shoes, I'm very good at playing!"),
        11: ".ue .i ku'i mi dasni lo mi cnino cutci",
        12: (".i mi na nitcu lo nu dasni lo kargu cutci doi .karlos. .i ki'u bo mi boltipyjvi so'iroi", "I don't need expensive shoes, Carlo... because I play a lot!"),
    },
    Q={1: {"type": "choice", "en": "Carlo and Liliana are playing basketball.", "jbo": ".i xu la .karlos. .e la .lilianas. cu lankyboijvi", "answers": [
            {"en": "Yes, that's true.", "jbo": "go'i", "correct": False},
            {"en": "No, that's not right.", "jbo": "na go'i", "correct": True}]},
       2: {"type": "choice", "en": "Why can Liliana score more goals than Carlo?", "jbo": ".i ki'u ma la .lilianas. cu jinga", "answers": [
            {"en": "Because she has expensive shoes.", "jbo": "ri ponse lo kargu cutci", "correct": False},
            {"en": "Because she plays a lot.", "jbo": "ri boltipyjvi so'iroi", "correct": True},
            {"en": "Because her dad is an expert player.", "jbo": "lo patfu be ri cu certu lo ka boltipyjvi", "correct": False}]}},
    D=[("line 9", ".i la .lilianas. cu lebna lo bolci gi'e renro ri fi lo kelci vorme", "\"score a goal\" is paraphrased as throwing the ball into the game-door (kelci vorme); no dictionary word for a goal")],
)

FIXES["batch2-04"] = dict(
    L={
        0: (".i la .marinas. cu zvati lo muzga fi'o kansa lo rirpa'u be ri", None),
        2: (".i su'o pixra cu zvati .i ri slabu", "There are some old photos."),
        3: "go'i .i ti pixra la .montreal.",
        5: ".i ko catlu .i ti pixra lo mikce tadni",
        6: "go'i .i ri zvati lo balcu'e be la .montreal.",
        9: ".i lo rirpa'u be la .marinas. cu catlu lo pixra gi'e smaji",
        13: ".i ri rirmamta mi",
        15: ".i tu'a lo mi lanzu cu mutce cinri",
    },
    Q={1: {"type": "choice", "en": "There are old photos in the museum.", "jbo": ".i xu lo muzga cu vasru lo slabu pixra", "answers": [
            {"en": "No, that's wrong.", "jbo": "na go'i", "correct": False},
            {"en": "Yes, that's true.", "jbo": "go'i", "correct": True}]},
       2: {"type": "choice", "en": "What does Marina notice about the picture?", "jbo": ".i la .marinas. cu viska ma vi lo pixra", "answers": [
            {"en": "The cars are all new.", "jbo": "ro lo karce cu cnino", "correct": False},
            {"en": "There are no cars.", "jbo": "no karce", "correct": True},
            {"en": "The photo is new.", "jbo": "lo pixra cu cnino", "correct": False}]}},
    D=[],
)

FIXES["batch2-05"] = dict(
    L={
        2: ".i mi nitcu tu'a lo mi taxfu",
        7: ".i mi djica tu'a lo crino taxfu",
        8: ".i xu do djica tu'a lo mi blanu taxfu",
        15: (".i go'i .i ji'a mi traji lo ka se nelci do kei fo lo do mensi", None),
    },
    Q={2: {"type": "choice", "en": "Why is Anna annoyed?", "jbo": ".i ki'u ma la .anas. cu fengu", "answers": [
        {"en": "Maria won't leave the bathroom.", "jbo": "la .marias. na cliva lo lumku'a", "correct": False},
        {"en": "Maria is wearing the dress Anna wanted.", "jbo": "la .marias. cu dasni lo taxfu poi la .anas. cu djica tu'a ke'a", "correct": True},
        {"en": "Maria is talking to her boyfriend.", "jbo": "la .marias. cu tavla lo pendo be ri", "correct": False}]}},
    D=[],
)

FIXES["batch2-06"] = dict(
    L={
        3: "go'i .e'o .i mi nitcu tu'a lo cnino taxfu",
        6: ".i mi nitcu tu'a lo cnino kosta",
        9: ".i mi ji'a nitcu tu'a lo cnino creka",
        10: ".i mapti .i do nitcu tu'a lo cnino taxfu ki'u ma",
        11: (".i mu'i lo nu mi ba cpedu lo nu gunka", "For a job interview."),
        12: ".i ri fasnu vi ma",
    },
    Q={1: {"type": "choice", "en": "Why does the man need new clothes?", "jbo": ".i ki'u ma lo nanmu cu nitcu tu'a lo cnino taxfu", "answers": [
            {"en": "He lost his old clothes.", "jbo": "ri pu cirko lo slabu taxfu", "correct": False},
            {"en": "He has a date.", "jbo": "ri penmi lo prami", "correct": False},
            {"en": "He has a job interview.", "jbo": "ri cpedu lo nu gunka", "correct": True}]},
       2: {"type": "choice", "en": "Where is the man's job interview?", "jbo": ".i ma stuzi lo nu cpedu", "answers": [
            {"en": "At a hospital.", "jbo": "lo spita", "correct": False},
            {"en": "At an office nearby.", "jbo": "lo jibni briju", "correct": False},
            {"en": "At the store he is shopping in.", "jbo": "lo zarci poi ri ca'o te vecnu vi ke'a", "correct": True}]}},
    D=[("line 11", ".i mu'i lo nu mi ba cpedu lo nu gunka", "\"job interview\" paraphrased as \"I will ask for work\"; no dictionary lujvo for an interview")],
)

FIXES["batch2-07"] = dict(
    L={
        2: ("coi .i .e'o mi djica lo nu pinxe lo birje", None),
        3: ".i do nanca li xo",
        5: ".i xu mi ka'e viska lo do se cmene karda .i xu do ponse lo do jaspu",
        7: ".oi .i lo mi jaspu cu zvati lo xotli",
        11: ".i ku'i mi makcu .i mi se jibri gi'e rirni re verba",
        12: ".u'u .i mi nitcu lo nu viska lo do se cmene karda",
        13: ".i do ka'e bevri ma poi se pinxe ku'o mi ca lo nu mi claxu lo karda",
    },
    Q={1: {"type": "choice", "en": "How old is Emilia?", "jbo": ".i la .emilias. cu nanca li xo", "answers": [
            {"en": "Twenty-five.", "jbo": "li remu", "correct": False},
            {"en": "Thirty-five.", "jbo": "li cimu", "correct": True},
            {"en": "Eighteen.", "jbo": "li pabi", "correct": False}]},
       2: {"type": "choice", "en": "The only drink Emilia can have without ID is...", "jbo": ".i la .emilias. ka'e pinxe ma po'o ca lo nu ri claxu lo karda", "answers": [
            {"en": "a beer", "jbo": "lo birje", "correct": False},
            {"en": "a glass of milk", "jbo": "lo kabri be lo ladru", "correct": True},
            {"en": "a glass of wine", "jbo": "lo kabri be lo vanju", "correct": False}]}},
    D=[("line 3", ".i lo nu do jmive cu nanca li xo", "nanca x1 is a TimeInterval; \"mi nanca li X\" for age is common usage but type-wise the duration is the life; kept the idiom"),
       ("line 4", ".i lo nu mi jmive cu nanca li cimu", "same age idiom as line 3"),
       ("line 8", ".i ku'i mi se cmene zoi gy. Emilia Rossi gy. .i lo nu mi jmive cu nanca li cimu", "same age idiom as line 3"),
       ("q 1", ".i lo nu la .emilias. cu jmive cu nanca li xo", "same age idiom as line 3")],
)

FIXES["batch2-08"] = dict(
    L={
        7: ".oi .i mi'a se ctuca ba lo re cacra",
        9: (".i ba lo re cacra la .djordjos. .e la .lorenas. cu se ctuca fo lo glico bangu", "Two hours later, Giorgio and Lorena are in English class."),
    },
    Q={2: {"type": "choice", "en": "Why did Lorena trick Giorgio about the exam?", "jbo": ".i ki'u ma la .lorenas. cu tcica la .djordjos.", "answers": [
        {"en": "She wanted him to fail.", "jbo": "djica lo nu ri fliba", "correct": False},
        {"en": "She wanted the teacher to laugh at him.", "jbo": "djica lo nu lo ctuca cu cmila ri", "correct": False},
        {"en": "She wanted to help him get ready.", "jbo": "djica lo nu sidju ri lo nu bredi", "correct": True}]}},
    D=[],
)

FIXES["batch2-09"] = dict(
    L={
        3: ".i lo purdi cu krasi ri",
        4: (".i la .roman. cu terpa .i ri catlu fi lo canko gi'e tirna", "Roma is scared. He looks out the window and listens."),
        12: "zoi gy. woof woof gy.",
        16: ".i .ei do kansa mi vi lo zdani .i do mi bandu",
    },
    Q={1: {"type": "choice", "en": "Who is Roma calling?", "jbo": ".i la .roman. cu fonta'a ma", "answers": [
            {"en": "His friend.", "jbo": "lo ri pendo", "correct": False},
            {"en": "His parents.", "jbo": "lo ri rirni", "correct": False},
            {"en": "The police.", "jbo": "lo pulji", "correct": True}]},
       2: {"type": "choice", "en": "The strange noise outside was...", "jbo": ".i lo cizra savru cu sance ma", "answers": [
            {"en": "the police", "jbo": "lo pulji", "correct": False},
            {"en": "his dog, Rex", "jbo": "lo ri gerku po'u la .reks.", "correct": True},
            {"en": "a thief", "jbo": "lo zerle'a", "correct": False}]}},
    D=[],
)

FIXES["batch2-10"] = dict(
    title={"en": "The Passport", "jbo": "lo jaspu"},
    speakers=[{"name": "Vladimir", "gender": "M"}, {"name": "Driver", "gender": "F"}],
    L={
        5: (".ua .i ri nenri lo karce", "Ah! It's in the taxi (the car)!"),
        6: (".i la .vladimir. cu sisku lo karce", "Vladimir looks for the car."),
        7: (".i ri viska lo jitro be lo karce", "He sees the driver."),
        13: "na go'i .i na ka'e go'i",
        16: ".i ki'e sai",
    },
    Q={1: {"type": "choice", "en": "Vladimir looks for the car and...", "jbo": ".i la .vladimir. cu sisku lo karce .i ri mo", "answers": [
            {"en": "doesn't see the driver", "jbo": "na viska lo karce jitro", "correct": False},
            {"en": "sees the driver", "jbo": "viska lo karce jitro", "correct": True},
            {"en": "goes home", "jbo": "klama lo zdani", "correct": False}]},
       2: {"type": "choice", "en": "Where was Vladimir's passport?", "jbo": ".i lo jaspu be la .vladimir. pu zvati ma", "answers": [
            {"en": "In his hand.", "jbo": "lo ri xance", "correct": True},
            {"en": "At his house.", "jbo": "lo ri zdani", "correct": False},
            {"en": "In the car.", "jbo": "lo karce", "correct": False}]}},
    D=[("line 5", ".ua .i ri nenri lo karce", "\"taxi\" (taksi) is only an experimental gismu, so the taxi is just \"lo karce\" (the car) throughout")],
)

FIXES["batch2-11"] = dict(
    L={
        1: "doi .sfetas. .i do traji lo ka se nelci mi kei fo lo mi tixnu",
        2: "doi mamta .i do se tixnu pa da po'o",
        7: ".i ti rirmamta mi",
        11: ".i ti rirpa'u mi .i la .alekseis.",
        13: ".i lo do lanzu cu barda",
        14: ".i ku'i mi se tixnu pa da po'o",
        16: ".uo .i ca bo mi se tixnu re da .i fi'i .olias. .i do ca cmima lo lanzu",
    },
    Q={2: {"type": "choice", "en": "After Olya was introduced to everyone...", "jbo": ".i ba lo nu la .olias. cu penmi ro da kei ma fasnu", "answers": [
        {"en": "Sveta's mother told her to leave.", "jbo": "lo mamta cu minde ri lo nu ri cliva", "correct": False},
        {"en": "Sveta's mother welcomed her to the family.", "jbo": "lo mamta cu rinsa ri", "correct": True},
        {"en": "She ran away quickly.", "jbo": "ri sutra bajra cliva", "correct": False}]}},
    D=[],
)

FIXES["batch2-12"] = dict(
    L={
        6: ".i xu do ponse lo do kulselgu'a",
        10: ".i xu do nitcu tu'a lo penbi",
        13: (".i mi ponse re pikta lo nu zgike ca lo cabycte", "I have two tickets for a concert tonight..."),
        17: (".ua .i go'i .i mi ka'e sidju do", "Ah... yes! I can help you!"),
    },
    Q={1: {"type": "choice", "en": "What is Luca asking?", "jbo": ".i la .lukas. cu retsku ma", "answers": [
            {"en": "If Selena feels well.", "jbo": "xu la .selenas. kanro", "correct": False},
            {"en": "If Selena wants to go to university.", "jbo": "xu do djica lo nu klama lo balcu'e", "correct": False},
            {"en": "If he can help Selena.", "jbo": "xu mi ka'e sidju do", "correct": True}]},
       2: {"type": "choice", "en": "What did Selena do?", "jbo": ".i la .selenas. pu zukte ma", "answers": [
            {"en": "She asked Luca for a pen.", "jbo": "cpedu lo penbi", "correct": False},
            {"en": "She invited Luca to a concert with her.", "jbo": "vi'ecpe la .lukas. lo nu zgike", "correct": True},
            {"en": "She went home to study.", "jbo": "klama lo zdani", "correct": False}]}},
    D=[],
)

FIXES["batch2-13"] = dict(
    L={
        1: ".i mi roroi citka lo donri sanmi vi lo mi jubme",
        2: ".i ku'i le ti cuxste cu vasru so'i xamgu",
        4: ".i mi djica lo nu citka lo burgere .e lo ga'arpatlu",
        5: ".i lo burgere vi ti cu mutce xamgu",
        7: ".i lo donri sanmi poi mi citka ke'a ca ro lo djedi",
        9: ".i lo burgere poi claxu lo rectu .e lo ga'arpatlu",
        13: ".i lo donri sanmi poi mi traji nelci ke'a cu burgere",
        14: ".i ri claxu lo rectu .e lo nanba .i ri vasru so'i stagi .e lo tamca",
    },
    Q={1: {"type": "choice", "en": "Renata usually eats lunch at home.", "jbo": ".i xu la .renatas. cu so'iroi citka lo donri sanmi vi lo zdani", "answers": [
            {"en": "Yes, that's true.", "jbo": "go'i", "correct": False},
            {"en": "No, that's not right.", "jbo": "na go'i", "correct": True}]},
       2: {"type": "choice", "en": "Renata is amused because Adriano's favourite lunch...", "jbo": ".i lo donri sanmi poi la .adrianos. cu traji nelci ke'a cu mo", "answers": [
            {"en": "is basically a salad.", "jbo": "salta", "correct": True},
            {"en": "has a lot of sugar.", "jbo": "vasru so'i sakta", "correct": False},
            {"en": "contains meat.", "jbo": "vasru lo rectu", "correct": False}]}},
    D=[("line 4", ".i mi djica lo nu citka lo burgere .e lo ga'arpatlu", "burgere is a fu'ivla (fuhivla.tsv), ga'arpatlu a lujvo; chosen over the earlier paraphrase \"nanba poi se nenri lo rectu\"")],
)

FIXES["batch2-14"] = dict(
    L={
        0: ".i la .braulios. cu zvati lo panka fi'o kansa la .adas. .e lo ri gerku",
        5: ".i ri so'iroi tinbe mi .i ku'i ri na roroi tinbe mi",
        10: ".i ca bo mi djica lo nu citka",
        11: ".i xu do djica lo nu citka lo do djasnuji",
        13: "doi .maks. .i ko na zukte ta",
        17: ".i ri so'iroi tinbe do .i ku'i ri na roroi tinbe do",
    },
    Q={1: {"type": "choice", "en": "After teaching her dog something new...", "jbo": ".i la .adas. cu mo ba lo nu ri ctuca lo gerku", "answers": [
            {"en": "Ada gives it something to eat.", "jbo": "dunda lo cidja lo gerku", "correct": True},
            {"en": "Ada gives it a toy.", "jbo": "dunda lo zilkei lo gerku", "correct": False},
            {"en": "Ada gives it some water.", "jbo": "dunda lo djacu lo gerku", "correct": False}]},
       2: {"type": "choice", "en": "What happened in the story?", "jbo": ".i ma fasnu", "answers": [
            {"en": "Braulio bought a toy for the dog.", "jbo": "la .braulios. cu te vecnu lo zilkei", "correct": False},
            {"en": "Ada's dog ate Braulio's sandwich.", "jbo": "lo gerku cu citka lo djasnuji pe la .braulios.", "correct": True},
            {"en": "Braulio decided to buy a dog too.", "jbo": "la .braulios. cu ji'a te vecnu lo gerku", "correct": False}]}},
    D=[],
)

FIXES["batch2-15"] = dict(
    L={
        7: ".i mi na ka'e facki lo du'u lo mi se nelci cukta cu zvati ma",
        8: ".i mi djica lo nu citka lo djasnuji",
        10: (".i .oi .i .ei mi ca klama lo ctuca kumfa", "And... oh no! I have to go to class now!"),
        17: ".i ki'u lo nu mi ca kansa do",
    },
    Q={1: {"type": "choice", "en": "Rebecca is looking for...", "jbo": ".i la .rebekas. ca sisku ma", "answers": [
            {"en": "her favourite book", "jbo": "lo ri se nelci cukta", "correct": True},
            {"en": "her best friend", "jbo": "lo ri traji pendo", "correct": False},
            {"en": "a bad day", "jbo": "lo xlali djedi", "correct": False}]},
       2: {"type": "choice", "en": "Why is Rebecca happy now?", "jbo": ".i ki'u ma la .rebekas. ca gleki", "answers": [
            {"en": "She got her book back.", "jbo": "ri pu cpacu lo ri cukta", "correct": False},
            {"en": "Eva gave her a sandwich.", "jbo": "la .evas. cu dunda lo djasnuji ri", "correct": False},
            {"en": "She is with her best friend now.", "jbo": "ri ca kansa lo ri traji pendo", "correct": True}]}},
    D=[],
)

FIXES["batch2-16"] = dict(
    SUBS=[(r"\.frantcesk\.", ".frantceskas.")],
    L={
        0: ".i la .frantceskas. cu zvati lo zdani fi'o kansa lo ri bruna po'u la .pietros.",
        3: ("go'i .i mi djica lo nu citka lo nanba .e lo mi cirla", "Yes, I want some bread with my cheese."),
        4: ".i xu do ka'e te vecnu pa dacti va'u mi",
        7: (".i mi pilno ri lo nu zbasu lo mi salta", "I'll use it for my salad."),
        10: ".i ji'a ci lurnanba",
        12: ".i ji'a lo jisra be lo narju",
        14: ".i ji'a lo ckafi .i .e'o",
        18: ".i mi djica tu'a pa dacti po'o .i lo nanba",
    },
    Q={1: {"type": "choice", "en": "Pietro asks...", "jbo": ".i la .pietros. cu retsku ma", "answers": [
            {"en": "if he can buy something for Francesca", "jbo": "xu mi ka'e te vecnu da va'u do", "correct": False},
            {"en": "if Francesca can buy one thing for him", "jbo": "xu do ka'e te vecnu pa dacti va'u mi", "correct": True},
            {"en": "if they will eat cheese together", "jbo": "xu mi'o kansa citka lo cirla", "correct": False}]},
       2: {"type": "choice", "en": "Francesca decided to...", "jbo": ".i la .frantceskas. cu jdice ma", "answers": [
            {"en": "make a cheese sandwich for Pietro", "jbo": "zbasu lo cirla djasnuji va'u la .pietros.", "correct": False},
            {"en": "stay home and send Pietro for bread", "jbo": "stali lo zdani gi'e rinka lo nu la .pietros. cu te vecnu lo nanba", "correct": True},
            {"en": "buy three croissants", "jbo": "te vecnu ci lurnanba", "correct": False}]}},
    D=[],
)

FIXES["batch2-17"] = dict(
    L={
        0: ".i la .kentas. cu zvati lo gusta fi'o kansa lo ri mamta",
        5: ".i mi ba gasnu lo nu do se spaji",
        6: ".i mi ji'a ba gasnu lo nu do se spaji",
        9: ".i lo mi pendo cu se cmene zo .kimuras. .i ri mapti do",
        14: ".i do ba'a mamta la .kentas.",
        16: ".i mi gleki lo nu penmi do .i mi'e .xirokos. .uedas. .i mi speni la .kentas.",
    },
    Q={1: {"type": "choice", "en": "Why is Kenta confused?", "jbo": ".i ki'u ma la .kentas. cu se cfipu", "answers": [
            {"en": "His mom didn't come.", "jbo": "lo mamta na klama", "correct": False},
            {"en": "His mom asked for a table for three.", "jbo": "lo mamta cu cpedu lo jubme be ci prenu", "correct": True},
            {"en": "His mom asked for three tables.", "jbo": "lo mamta cu cpedu ci jubme", "correct": False}]},
       2: {"type": "choice", "en": "Why is Kenta's mom surprised?", "jbo": ".i ki'u ma lo mamta cu se spaji", "answers": [
            {"en": "She didn't know Kenta was married.", "jbo": "ri na djuno lo du'u la .kentas. speni", "correct": True},
            {"en": "She is happy to see Hiroko.", "jbo": "ri gleki lo nu viska la .xirokos.", "correct": False},
            {"en": "The food arrived quickly.", "jbo": "lo cidja cu sutra klama", "correct": False}]}},
    D=[],
)

FIXES["batch2-18"] = dict(
    SUBS=[(r"\.frantcesk\.", ".frantceskos.")],
    L={
        3: "go'i .e'o .i .ei mi te vecnu da mu'i lo nu jukpa lo vanci sanmi",
        6: ".i .ei mi jukpa lo mapti vanci sanmi mu'i lo jbedetnunsla",
        7: ".i mi nelci lo jbedetnunsla",
        8: ".i lo cabdei cu jbedetri la .frantceskos.",
        14: ".i la .frantceskos. cu nanmu .i ri mutce se funca lo xamgu",
        18: ".i .uizo'o .i la .frantceskos. cu mlatu .i ri mutce se funca lo xamgu",
    },
    Q={1: {"type": "choice", "en": "Why is Michela buying salmon?", "jbo": ".i ki'u ma la .mikelys. cu te vecnu lo salmone", "answers": [
        {"en": "It's cheap.", "jbo": "ri tolkargu", "correct": False},
        {"en": "Francesco never eats meat.", "jbo": "la .frantceskos. noroi citka lo rectu", "correct": False},
        {"en": "Francesco loves fish.", "jbo": "la .frantceskos. cu mutce nelci lo finpe", "correct": True}]}},
    D=[("line 1", ".i .u'u doi nanmu", "\"sir\" has no polite equivalent; \"doi nanmu\" (O man) is blunt but grammatical")],
)

# cipra: x1 test, x2 property tested, x3 subject -> the student is "te cipra"
FIXES["batch2-08"]["L"].update({
    3: ".i mi'a na te cipra",
    4: "go'i .i mi'a te cipra ca lo cabdei doi .djordjos.",
})

FIXES["batch2-19"] = dict(
    title={"en": "The Basketball Player", "jbo": "lo lankyboijvi"},
    L={
        2: (".i mi'o nitcu lo nu te vecnu lo ladru .e lo rismi .e lo dembi", None),
        3: ("doi .rodolfos. .i ko catlu tu", "Rodolfo! Look over there!"),
        5: (".i tu du lo lankyboijvi poi do traji nelci", "That's your favourite basketball player!"),
        6: (".i na go'i", "No, it isn't!"),
        7: (".i go'i .i ko tavla tu", "Yes, it is! Talk to him!"),
        8: ".i mi dukse lo ka terpa",
        10: ".i lo lankyboijvi cu catlu la .rodolfos. .e la .barbaras.",
    },
    Q={1: {"type": "choice", "en": "Barbara says the player is coming towards them.", "jbo": ".i xu la .barbaras. cu cusku lo se du'u lo lankyboijvi cu klama ri", "answers": [
            {"en": "No, that's not right.", "jbo": "na go'i", "correct": False},
            {"en": "Yes, that's true.", "jbo": "go'i", "correct": True}]},
       2: {"type": "choice", "en": "The player wanted to tell Rodolfo that...", "jbo": ".i lo lankyboijvi cu djica lo nu cusku ma", "answers": [
            {"en": "he likes Rodolfo's shoes", "jbo": "lo se du'u ri nelci lo cutci pe la .rodolfos.", "correct": True},
            {"en": "they shouldn't stare", "jbo": "lo se du'u .ei ri na catlu", "correct": False},
            {"en": "he could sign an autograph", "jbo": "lo se du'u ri ka'e ciska lo cmene", "correct": False}]}},
    D=[("line 8", ".i mi dukse lo ka terpa", "\"nervous\" rendered as \"too afraid\"; no exact gismu")],
)

FIXES["batch2-20"] = dict(
    title={"en": "My Favourite Game", "jbo": "lo nunkei poi mi traji nelci"},
    L={
        4: ".i mi mutce zukcfu ca lo cabdei",
        6: "go'i .i lo do kumfa cu mutce na'e lumci",
        11: ".i'e .i mi se slabu pa mutce zdile nunkei",
        12: ".ue .i ma nunkei",
        13: ".i ri traji lo ka se nelci mi",
        15: (".i le ti nunkei cu sampu", "This game is simple..."),
        17: ".i ku'i .ei mi cliva .i mi mutce zukcfu ca lo cabdei",
    },
    Q={2: {"type": "choice", "en": "What was Samuele's \"favourite game\"?", "jbo": ".i la .samuel. cu traji nelci ma", "answers": [
        {"en": "Having his brother help clean his room.", "jbo": "lo nu lo bruna cu sidju fi lo nu lumci lo kumfa", "correct": True},
        {"en": "Hide-and-seek.", "jbo": "lo nu mipri sisku", "correct": False},
        {"en": "Playing outside.", "jbo": "lo nu kelci vi lo bartu", "correct": False}]}},
    D=[],
)

FIXES["batch2-21"] = dict(
    L={
        1: (".i mo doi .martcelos. .i mi ba te cipra ca lo cabdei .i lo cipra cu vajni", "What is it, Marcello? I have an exam today! An important one!"),
        2: ".i ti'u li xa .i ki'u ma do na sipna",
        5: "doi .martcelos. .i xu do jukpa le ti cerni sanmi va'u mi",
        7: (".i mi djuno lo du'u do ba te cipra ca lo cabdei", "I know you have an exam today..."),
        8: ".i do nitcu lo nu citka lo xamgu cerni sanmi poi vasru lo gurni .e lo ladru .e lo nanba .e lo jisra",
        11: (".i ku'i mi na ka'e citka ca lo cabna .i .ei mi cliva", "But I can't eat now! I have to go!"),
        12: ".i mi djica lo nu pinxe lo ckafi po'o",
        15: (".i no ckafi cu stali .i ku'i mi ka'e zbasu lo ckafi", "There's no more coffee, but I can make some!"),
        16: "doi .martcelos. .i mi nitcu tu'a lo mi ckafi",
        18: ".i mi ji'a nitcu tu'a lo ckafi",
    },
    Q={1: {"type": "choice", "en": "What's Marcello's surprise?", "jbo": ".i lo spaji poi la .martcelos. cu gasnu ke'a cu mo", "answers": [
            {"en": "He made breakfast for Samanta.", "jbo": "lo nu ri jukpa lo cerni sanmi va'u la .samantas.", "correct": True},
            {"en": "He knows the test answers.", "jbo": "lo nu ri djuno lo du'u ma kau danfu lo cipra", "correct": False},
            {"en": "He renovated the kitchen.", "jbo": "lo nu ri cnino zbasu lo jupku'a", "correct": False}]},
       2: {"type": "choice", "en": "Marcello woke up very early, so...", "jbo": ".i la .martcelos. pu mutce cerni cikna .i se ki'u bo ri mo", "answers": [
            {"en": "he could study", "jbo": "tadni", "correct": False},
            {"en": "he fell asleep in the kitchen", "jbo": "sipna vi lo jupku'a", "correct": False},
            {"en": "he drank all the coffee", "jbo": "pinxe ro lo ckafi", "correct": True}]}},
    D=[],
)

FIXES["batch2-22"] = dict(
    L={
        4: (".i lo rirni be lo mi rirni cu ponse ri", "They're my grandparents'."),
        8: ".i lu mi zgipli lo jgita gi'e pensi do li'u",
        18: (".i ki'u lo nu mi'o ca ponse lo samxa'a", "Because now we have email!"),
    },
    Q={1: {"type": "choice", "en": "In the letter, Rosanna's grandfather says he...", "jbo": ".i lo rirpa'u be la .rozanas. cu mo", "answers": [
            {"en": "plays the guitar and thinks of Maria", "jbo": "zgipli lo jgita gi'e pensi la .marias.", "correct": True},
            {"en": "buys flowers for Maria", "jbo": "te vecnu lo xrula va'u la .marias.", "correct": False},
            {"en": "reads letters", "jbo": "tcidu lo xatra", "correct": False}]},
       2: {"type": "choice", "en": "Why doesn't Armando write love letters?", "jbo": ".i ki'u ma la .armandos. na ciska lo prami xatra", "answers": [
            {"en": "He doesn't love her.", "jbo": "ri na prami", "correct": False},
            {"en": "He can send email instead.", "jbo": "ri ka'e samymri", "correct": True},
            {"en": "He's too busy playing guitar.", "jbo": "ri dukse lo ka zgipli lo jgita", "correct": False}]}},
    D=[],
)

FIXES["batch2-23"] = dict(
    L={
        2: ".i ca lo jefyfa'o",
        4: ".i ko denpa .i lo mi briju fonxa cu ca'o se sance",
        5: "coi",
        10: ".i do djica ma .i do na slabu mi",
        11: ".i ku'i do slabu mi",
        12: ".i do mutce slabu mi",
        14: ".oi .i mi ba fonta'a lo pulji",
        15: ("doi .leonid. .i na nitcu lo nu do fonta'a lo pulji .i mi do jatna", "Leonid, no need to call the police. I'm your boss."),
        18: ".oi .i .u'u .i mi di'a gunka",
    },
    Q={1: {"type": "choice", "en": "Leonid doesn't know the woman on the phone.", "jbo": ".i xu lo ninmu na slabu la .leonid.", "answers": [
            {"en": "Yes, that's true.", "jbo": "go'i", "correct": True},
            {"en": "No, that's not right.", "jbo": "na go'i", "correct": False}]},
       2: {"type": "choice", "en": "Who is the stranger?", "jbo": ".i lo prenu poi na slabu la .leonid. cu du ma", "answers": [
            {"en": "Leonid's mother.", "jbo": "lo mamta be la .leonid.", "correct": False},
            {"en": "His friend from college.", "jbo": "lo pendo be la .leonid.", "correct": False},
            {"en": "Leonid's boss.", "jbo": "lo jatna be la .leonid.", "correct": True}]}},
    D=[],
)

FIXES["batch2-24"] = dict(
    L={
        0: ".i la .lilis. cu zvati lo taxfu zarci fi'o kansa lo ri pendo po'u la .zaris.",
        11: ".i lo kosta cu se jdima lo meryru'u be li munono",
        13: "doi .zaris. .i do na nitcu tu'a lo cnino kosta",
        14: ".i ku'i doi .lilis. lo cabdei cu jbedetri mi",
        17: ("go'i .i mo", "Yes... and?"),
        18: (".i lo mi mapti seldu'a cu mo", "And the perfect gift for me is...?"),
        19: ".i lo kosta poi tolkargu",
    },
    Q={1: {"type": "choice", "en": "How much does the jacket cost?", "jbo": ".i lo kosta cu se jdima lo meryru'u be li xo", "answers": [
            {"en": "five dollars", "jbo": "li mu", "correct": False},
            {"en": "five hundred dollars", "jbo": "li munono", "correct": True},
            {"en": "five million", "jbo": "li mu ki'o ki'o", "correct": False}]},
       2: {"type": "choice", "en": "Zari's birthday gift from Lili will be...", "jbo": ".i la .lilis. ba dunda ma la .zaris.", "answers": [
            {"en": "an expensive red jacket", "jbo": "lo kargu xunre kosta", "correct": False},
            {"en": "a pile of money", "jbo": "lo so'i jdini", "correct": False},
            {"en": "a cheap jacket", "jbo": "lo kosta poi tolkargu", "correct": True}]}},
    D=[],
)

FIXES["batch2-25"] = dict(
    title={"en": "Doctor Eddy", "jbo": "lo mikce po'u la .edis."},
    L={
        13: ".i le ta nanmu ca nitcu tu'a lo mikce",
        15: ".e'o .i le nanmu cu nitcu lo nu do sidju ri",
    },
    Q={1: {"type": "choice", "en": "The man who needs help is near the...", "jbo": ".i lo nanmu poi nitcu lo nu sidju cu jibni ma", "answers": [
            {"en": "station", "jbo": "lo tcana", "correct": False},
            {"en": "tomatoes", "jbo": "lo tamca", "correct": False},
            {"en": "milk", "jbo": "lo ladru", "correct": True}]},
       2: {"type": "choice", "en": "What happened in the story?", "jbo": ".i ma fasnu", "answers": [
            {"en": "Eddy started med school.", "jbo": "la .edis. co'a tadni lo mikce saske", "correct": False},
            {"en": "Eddy said he was a doctor to impress a woman.", "jbo": "la .edis. cu cusku lo se du'u ri mikce", "correct": True},
            {"en": "Eddy saved a man.", "jbo": "la .edis. cu sidju pa nanmu", "correct": False}]}},
    D=[],
)

FIXES["batch2-26"] = dict(
    L={
        4: ".i mi nitcu tu'a lo mi penbi",
        7: ".i xu do nitcu tu'a lo penbi",
        8: ("go'i .i mi ba te cipra .i lo cipra cu vajni", "Yes! I have an exam! An important one!"),
        14: ".i do na nitcu tu'a lo penbi ca lo cipra",
        16: ".i ki'u lo nu lo cipra cu fasnu vi lo skami",
    },
    Q={1: {"type": "choice", "en": "When does Lorenzo use his favourite pen?", "jbo": ".i la .lorentsos. cu pilno lo ri se nelci penbi ca ma", "answers": [
            {"en": "During all exams.", "jbo": "ca ro lo cipra", "correct": True},
            {"en": "Never.", "jbo": "noroi", "correct": False},
            {"en": "During two exams.", "jbo": "ca re cipra", "correct": False}]},
       2: {"type": "choice", "en": "Why is Marco amused?", "jbo": ".i ki'u ma la .markos. cu cmila", "answers": [
            {"en": "The exam is on a computer.", "jbo": "lo cipra cu fasnu vi lo skami", "correct": True},
            {"en": "Lorenzo is a comedian.", "jbo": "la .lorentsos. cu xajmi prenu", "correct": False},
            {"en": "He found the pen.", "jbo": "ri pu facki lo du'u lo penbi cu zvati ma", "correct": False}]}},
    D=[],
)

FIXES["batch2-27"] = dict(
    L={
        1: "doi .djoeles. .i xu .ei do ca zgipli lo jgita",
        3: (".i mi djica lo nu xamgu zgipli lo jgita", "I want to play the guitar well."),
        4: ".i mi djica tu'a lo se prami",
        5: (".i lo ninmu cu prami ma", "...and who do women love?"),
        6: "lo nanmu poi zgipli lo jgita",
        7: ".i xu .ei do zgipli ca lo nicte",
        9: ".i'e .i mi ba sisti lo nu zgipli lo jgita",
        10: ("ki'e .i ko xamgu sipna", "Thanks. Sleep well."),
        11: (".i ko ji'a xamgu sipna", "You too, sleep well."),
        12: ".i la .djoeles. cu co'a zgipli lo pipno",
        14: ".i ma .i mi na ka'e zgipli lo jgita",
        15: ".i ku'i mi ka'e zgipli lo pipno .i xu go'i",
        16: ".i lo ninmu cu prami lo nanmu poi zgipli lo pipno",
    },
    Q={1: {"type": "choice", "en": "Why does Gioele want to play the guitar?", "jbo": ".i ki'u ma la .djoeles. cu djica lo nu zgipli lo jgita", "answers": [
            {"en": "He thinks women like guitar players.", "jbo": "jinvi lo du'u lo ninmu cu nelci lo nanmu poi zgipli lo jgita", "correct": True},
            {"en": "He wants to play for his friend.", "jbo": "djica lo nu zgipli va'u lo pendo", "correct": False},
            {"en": "He wants to play before sleep.", "jbo": "djica lo nu zgipli pu lo nu sipna", "correct": False}]},
       2: {"type": "choice", "en": "Since Tommaso didn't want him to play guitar...", "jbo": ".i ki'u lo nu la .tomasos. na djica lo nu la .djoeles. cu zgipli lo jgita kei ma fasnu", "answers": [
            {"en": "Gioele went to bed.", "jbo": "la .djoeles. cu sipna", "correct": False},
            {"en": "Gioele started playing the piano.", "jbo": "la .djoeles. cu co'a zgipli lo pipno", "correct": True},
            {"en": "Gioele left the house.", "jbo": "la .djoeles. cu cliva lo zdani", "correct": False}]}},
    D=[],
)


def ch(jbo, en, answers):
    """Compact builder for a choice question: answers = [(jbo, en, correct)]."""
    return {"type": "choice", "en": en, "jbo": jbo,
            "answers": [{"en": a_en, "jbo": a_jbo, "correct": c} for a_jbo, a_en, c in answers]}


FIXES["batch2-28"] = dict(
    L={
        2: ".i mi na kakne lo ka co'a sipna",
        3: ".i xu do djica lo nu pinxe lo kabri be lo ladru",
        9: ".i lo skari be lo bitmu pe lo mi'o kumfa",
        14: ".i do nelci lo mo skari",
        16: ".oi .i ca bo mi na kakne lo ka co'a sipna",
    },
    Q={1: ch(".i ki'u ma ri na ka'e tirna lo zgike", "Why can't they listen to music?", [
            ("lo sance tutci cu spofu", "Their speaker is broken.", False),
            ("ri na ba sipna", "They won't sleep afterwards.", False),
            ("ri na djica lo nu lo cifnu cu cikna", "They don't want to wake the baby.", True)]),
       2: ch(".i ma fasnu", "What happened (to Anatoliy)?", [
            ("la .anatolis. cu cikygau lo cifnu", "He woke the baby.", False),
            ("la .anatolis. cu sidju la .ritas. lo nu co'a sipna .i ku'i ri na kakne lo ka co'a sipna", "He helped Rita fall asleep, but then he couldn't.", True),
            ("la .anatolis. cu punji lo cinta lo bitmu", "He painted the walls.", False)])},
    D=[],
)

FIXES["batch2-29"] = dict(
    L={
        2: ".i ri se cmene zo .metes.",
        7: ".i ri se bangu lo bangu pe la .turkias. .e lo dotco bangu .e lo glico bangu",
        8: ".i ji'a ri stati",
        9: ".i ri stati gi'e clani gi'e mutce melbi",
    },
    Q={1: ch(".i la .metes. cu mo", "How does Zari describe Mete?", [
            ("cmalu gi'e ponse lo pelxu kerfa", "short and blond", False),
            ("melbi je cladu", "handsome and loud", False),
            ("stati je clani", "intelligent and tall", True)]),
       2: ch(".i la .zaris. pu na djuno ma", "What didn't Zari realize?", [
            ("lo du'u lo cutci pe la .metes. na melbi", "that Mete's shoes were ugly", False),
            ("lo du'u la .metes. cu trixe ri", "that Mete was behind her", True),
            ("lo du'u ri ba te cipra ca lo cabdei", "that she had a test today", False)])},
    D=[("line 7", ".i ri se bangu lo bangu pe la .turkias. .e lo dotco bangu .e lo glico bangu",
        "no dictionary word for Turkish (tirkoi was not a valid lujvo); paraphrased as \"the language of Türkiye\"")],
)

FIXES["batch2-30"] = dict(
    L={
        2: ".i xu do ba zukte da ca lo jefyfa'o",
        3: ".i mi ponse pa pikta be fi lo nu zgike ca lo xavdei",
        4: (".i lo zgigri cu cnino .i ri se cmene lu lo mensi li'u", "It's a new band, \"The Sisters\"."),
        7: ".i do ba zukte ma ca lo jefyfa'o",
        9: "go'i .i pa boltipterjvi ca lo xavdei",
        11: ".i .a'o lo boltipterjvi cu zdile do",
        12: ".i ca lo xavdei vanci la .paulos. cu zvati lo nu zgike",
        13: ".i lo nu zgike cu cfari",
        15: ".i xu ta du la .vitorias.",
        16: ".i ba lo nu zgike ku la .paulos. cu tavla la .vitorias.",
        17: ".i boltipterjvi .ianai",
        19: ".i do mutce certu lo ka zgipli lo jgita",
    },
    Q={1: ch(".i la .paulos. cu retsku ma", "What does Paolo want to know?", [
            ("lu xu do nelci lo zgike li'u", "what music Vittoria likes", False),
            ("lu xu do djica lo nu klama lo nu zgike li'u", "if Vittoria wants to go to a concert", True),
            ("lu xu do gunka ca lo xavdei li'u", "if Vittoria works on Saturday", False)]),
       2: ch(".i ki'u ma la .paulos. cu se cfipu", "Why is Paolo confused?", [
            ("la .vitorias. cu fonta'a ri", "Vittoria is calling him.", False),
            ("lo nu zgike na pu cfari", "The show hasn't started.", False),
            ("la .vitorias. cu zgipli lo jgita ca lo nu zgike", "Vittoria is the band's guitarist.", True)])},
    D=[],
)

FIXES["batch2-31"] = dict(
    title={"en": "Grandma's New Phone", "jbo": "lo cnino fonxa po lo rirmamta"},
    L={
        0: "doi rirmamta .i mi ponse lo seldu'a va'u do",
        1: ".i xu va'u mi",
        4: ".i le ti fonxa cu zmadu fi lo ka xamgu",
        5: ".i ri vasru lo kacma",
        6: ".i xu lo fonxa poi vasru lo kacma",
        9: "go'i .i ri fonskami",
        12: ".i do ka'e samymri gi'e tadni lo bangu",
        17: ".i xu mi ka'e pilno ri lo nu fonta'a do",
    },
    Q={1: ch(".i sepi'o le fonxa do ka'e mo", "Roberto says with this phone you can...", [
            ("catlu lo skina .e lo tcima", "watch movies and check the weather", False),
            ("tirna lo zgike gi'e catlu lo tivni", "listen to music and watch TV", True),
            ("ciska lo xatra gi'e kelci", "write letters and play games", False)]),
       2: ch(".i lo rirmamta cu gleki ki'u lo nu ri ka'e mo sepi'o lo cnino fonxa", "Grandma is glad her new phone also lets her...", [
            ("zbasu lo ckafi", "make coffee", False),
            ("cikygau ri", "wake her up", False),
            ("fonta'a lo selrirbe'a be ri", "call her grandson", True)])},
    D=[("line 19", ".ua .i ri mutce stati fonxa .i ki'e",
        "pun on \"smart phone\" kept with stati (talented); fonskami is the dictionary word for smartphone")],
)

FIXES["batch2-32"] = dict(
    title={"en": "The School Party", "jbo": "lo nunsla pe lo ckule"},
    L={
        3: ".i pa nunsla cu fasnu vi lo ckule .i ri vasru lo zgike",
        6: ".i mi na djica lo nu dansu vi lo crane be lo drata",
        9: ".i mi ba ctuca do fo lo ka dansu",
        14: "doi mamta .i .ei do kansa mi lo ka klama lo nunsla",
        15: ".i ca lo nu mi zvati lo mlana be do",
    },
    Q={1: ch(".i ca lo nu la .fabios. cu dansu kansa lo mamta kei la .fabios. cu mo", "When dancing with his mother, Fabio...", [
            ("simlu lo ka xamgu dansu", "feels like a very good dancer", True),
            ("na ka'e dansu", "feels he can't dance", False),
            ("jinvi lo du'u ri .e lo mamta na ka'e kansa dansu", "thinks they can't dance together", False)])},
    D=[],
)

FIXES["batch2-33"] = dict(
    title={"en": "I Want a Dog", "jbo": "mi djica tu'a lo gerku"},
    L={
        2: ".i mi djica tu'a lo gerku",
        3: ".i xu lo gerku .i la'e di'u nitcu lo nu mutce gunka",
        8: ".i ji'a lo palta",
        10: ".i ji'a lo canko",
        14: ".i ca bo mi'a ka'e te vecnu lo gerku va'u do",
        16: ".i mi na djica tu'a lo gerku",
    },
    Q={1: ch(".i pu lo nu la .sacas. cu ponse lo gerku kei .ei ri mo", "Before getting a dog, Sasha has to...", [
            ("lumci lo kumfa .e lo jupku'a", "clean his room and kitchen", False),
            ("dunda lo djacu lo spati gi'e jukpa", "water plants and cook", False),
            ("lumci lo karce .e lo palta .e lo canko", "wash the car, dishes and windows", True)]),
       2: ch(".i ki'u ma la .sacas. na djica tu'a lo gerku", "Why doesn't Sasha want a dog?", [
            ("ri tatpi gi'e na djica lo nu fuzme", "He's tired and doesn't want more responsibility.", True),
            ("ri ponse vo gerku", "He already has four dogs.", False),
            ("lo nu ponse lo gerku na cinri", "Having a dog is boring.", False)])},
    D=[],
)

FIXES["batch2-34"] = dict(
    L={
        2: (".i xu do djica lo nu mi'o citka lo pitnanba", "Do you want to eat some pizza together?"),
        6: (".i so'i prenu cu zvati ta .i la'e di'u dukse", "There are so many people there. Too many."),
        14: (".i .ei mi te vecnu lo jbedetri seldu'a be fi do", "I need to buy a birthday present for you..."),
        19: (".i do zvati ti mu'i lo nu do te vecnu lo jbedetri seldu'a be fi mi", "You're here to buy me a birthday present!"),
    },
    Q={1: ch(".i ki'u ma la .margaritas. cu xebni lo barda zarci", "Why does Margarita hate the mall?", [
            ("ri roroi ganlo", "It's always closed.", False),
            ("ri dukse lo ka darno", "It's too far.", False),
            ("so'i prenu cu zvati ri .i la'e di'u dukse", "There are too many people.", True)]),
       2: ch(".i ki'u ma la .sfetas. cu jinvi lo du'u la .margaritas. cu xamgu pendo", "Why does Sveta think Margarita is a good friend?", [
            ("ri na nelci lo prenu", "She doesn't like people.", False),
            ("ri jukpa lo vanci sanmi", "She cooks her dinner.", False),
            ("la .margaritas. cu klama lo barda zarci mu'i lo nu te vecnu lo seldu'a be fi la .sfetas.", "She went to the mall to buy her a present.", True)])},
    D=[],
)

FIXES["batch2-35"] = dict(
    title={"en": "A Trip to Paris", "jbo": "lo nu klama la paris."},
    L={
        0: (".i la .pavel. .e la .kristinas. cu zvati la paris. ca lo nalgundei", "Pavel and Kristina are in Paris on holiday."),
        6: ".i mi nelci lo kafybarja .e lo bartu jubme",
        12: ".i lo zdani mo .i lo gunka mo",
        16: ".i ri nitcu tu'a lo djabei",
    },
    Q={1: ch(".i la .kristinas. cu retsku ma", "What does Kristina ask Pavel?", [
            ("lu xu do kanro li'u", "Are you OK?", False),
            ("lu xu do birti li'u", "Are you sure?", True),
            ("lu xu do zvati lo zdani li'u", "Are you home?", False)]),
       2: ch(".i la .pavel. .e la .kristinas. cu jdice ma", "What did Pavel and Kristina decide?", [
            ("lo du'u ri ba klama la paris. ca ro lo nalgundei", "To go to Paris every holiday.", False),
            ("lo du'u ri ba xabju la paris.", "To live in Paris.", True),
            ("lo du'u ri ba te vecnu lo gusta", "To buy a restaurant.", False)])},
    D=[("line 6", ".i mi nelci lo kafybarja .e lo bartu jubme",
        "\"terraces\" paraphrased as outdoor tables (bartu jubme); no dictionary word")],
)

FIXES["batch2-36"] = dict(
    L={
        0: ".i mi nitcu tu'a lo jdini",
        2: ".i mu'i lo nu klama lo kindi'u",
        3: ".i .ei do tadni mu'i lo nu do ba te cipra",
        10: ".i mi nitcu tu'a lo drata jdini",
        11: ".i ri banzu lo nu do klama lo ckusro sepi'o lo sorprekarce",
        14: ".i ti mu'i lo nu do te vecnu lo cidja be lo gerku",
        18: "na go'i .i mi nitcu lo nu pilno lo mi karce",
        20: ".i mu'i lo nu klama lo kindi'u",
    },
    Q={1: ch(".i lo mamta cu cusku ma", "What does Giovanni's mom say?", [
            ("lo se du'u ri djica lo nu klama lo ckusro", "She wants to go to the library.", False),
            ("lo se du'u lo jdini cu banzu lo nu klama lo ckusro sepi'o lo sorprekarce", "It's enough for the bus to the library.", True),
            ("lo se du'u lo jdini cu banzu lo nu klama lo kindi'u", "It's enough for the cinema.", False)]),
       2: ch(".i ki'u ma la .djovanis. na ka'e pilno lo karce", "Why can't Giovanni use his mom's car?", [
            ("ri se cikre", "It's being repaired.", False),
            ("lo mamta cu pilno ri lo nu klama lo kindi'u", "She's using it to go to the cinema.", True),
            ("lo mamta cu klama lo ckusro", "She's going to the library.", False)])},
    D=[],
)

FIXES["batch2-37"] = dict(
    L={
        1: ".i ti du lo mi'o xotli",
        7: (".i mi djica lo nu cadzu gi'e catlu ti", "I want to walk around and see this place!"),
        8: ".i mi ji'a djica lo nu klama sepi'o lo xirma",
        17: ".i vi lo brabaktu pe lo xotli",
        18: "go'i .i ba bo mi'o catlu lo skina vi lo mi'o kumfa",
    },
    Q={1: ch(".i la .klaudias. cu djica ma", "What does Claudia want to do?", [
            ("lo nu xruti lo zdani", "Go back home.", False),
            ("lo nu cadzu gi'e catlu", "Walk around and see the sights.", True),
            ("lo nu klama lo drata xotli", "Go to another hotel.", False)]),
       2: ch(".i la .diegos. .e la .klaudias. ca ba mo", "What are Diego and Claudia going to do now?", [
            ("pinxe lo vlipa se pinxe", "Have energy drinks.", False),
            ("surla ca lo vanci gi'e catlu ca lo bavlamdei", "Have a relaxing evening and see the sights tomorrow.", True),
            ("limna lo xamsi gi'e cpare", "Swim at the beach and climb.", False)])},
    D=[("line 0", ".i la .klaudias. .e la .diegos. cu zvati lo gugde panka", "\"national park\" as gugde panka (country-park) tanru; no dictionary lujvo"),
       ("line 5", ".i do djica lo nu pa moi zukte ma", "\"do first\" as pa moi zukte; alternative: lo nu zukte ma pu lo drata")],
)

FIXES["batch2-38"] = dict(
    L={
        5: "go'i .i ri ji'a vasru lo kindi'u",
        6: ".i ji'a lo itlo gusta poi do nelci",
        16: ".i ku'i ri se cmene lo drata cmene",
        18: ".i xu do djica lo nu klama lo cnino .ianai barda zarci",
    },
    Q={1: ch(".i lo barda zarci cu mo", "The mall...", [
            ("du lo slabu gi'e se cmene lo cnino cmene", "is the same one, with a new name", True),
            ("ba co'a ganlo", "is closing down", False),
            ("cnino gi'e zvati lo drata tcadu", "is new but in another town", False)]),
       2: ch(".i le re prenu ba mo", "What are they going to do?", [
            ("klama lo barda zarci vi la .italias.", "Go to a mall in Italy.", False),
            ("catlu lo skina gi'e citka lo itlo cidja", "Go to the movies and eat Italian food at the mall.", True),
            ("stali lo zdani", "Stay home and order food.", False)])},
    D=[("line 18", ".i xu do djica lo nu klama lo cnino .ianai barda zarci",
        "English scare quotes rendered with the attitudinal .ianai (disbelief) after cnino")],
)

FIXES["batch2-39"] = dict(
    title={"en": "Happy Birthday!", "jbo": ".a'o gleki jbedetri"},
    L={
        0: ".i .a'o gleki jbedetri doi dirba",
        1: ".i lo cabdei na jbedetri mi",
        2: "doi .dimas. .i lo cabdei cu jbedetri do",
        3: (".i mi na ba salci lo mi jbedetri", "I won't celebrate my birthday..."),
        4: (".i ja'o mi na ba nanca li cino", "...so I won't turn thirty!"),
        7: (".i lo cabdei na jbedetri do", "Today isn't your birthday..."),
        8: (".i ja'o no seldu'a .e no nunsla .e no titnanba", "...so no presents, no party and no cake."),
        10: ".i mi ba ralte lo seldu'a va'u mi",
        14: ".i ki'u lo nu lo cabdei na jbedetri do",
        19: ".i'e .i lo cabdei cu jbedetri mi",
        20: (".i ku'i mi djica lo nu reso laktergu'i po'o cu zvati lo titnanba", "But I only want twenty-nine candles on the cake."),
    },
    Q={1: ch(".i la .dimas. cu cusku ma", "What does Dima mean?", [
            ("lo se du'u ri ba badri", "He'll be sad.", False),
            ("lo se du'u lo ri jbedetri cu bavlamdei", "His birthday is tomorrow.", False),
            ("lo se du'u ganai ri na salci gi ri na ba nanca li cino", "If they don't celebrate, he won't turn 30.", True)]),
       2: ch(".i la .dimas. cu jdice lo du'u ri ba mo", "Dima decided to...", [
            ("salci lo ri cinomoi jbedetri gi'e pilno reso laktergu'i po'o", "celebrate his 30th but with only 29 candles", True),
            ("te vecnu lo skami va'u lo ri speni", "buy his wife a computer", False),
            ("sisti lo nu salci", "cancel the party", False)])},
    D=[("line 0", ".i .a'o gleki jbedetri doi dirba", "\"Happy birthday\" as a gleki jbedetri tanru calque; no established Lojban formula"),
       ("line 4", ".i ja'o lo nu mi jmive na ba nanca li cino", "same age idiom as batch2-07 (nanca x1 is a TimeInterval); the original ganai/gi split across two lines did not parse, so it became two sentences joined by ja'o"),
       ("q 1 answer 2", "lo se du'u ganai ri na salci gi lo nu ri jmive na ba nanca li cino", "same age idiom")],
)

FIXES["batch2-40"] = dict(
    L={
        2: ".i pa vajni boltipterjvi cu se tivni",
        4: ".i ku'i mi nitcu tu'a lo jdini",
        7: ".i mi nitcu tu'a lo jdini mu'i lo nu mi te vecnu lo pitnanba",
        9: ".i mi ji'a djica tu'a pa cnino samselkei",
        18: ".i lo boltipterjvi pu mutce xamgu",
    },
    Q={1: ch(".i ki'u ma la .djunior. cu se spaji", "Why is Junior surprised?", [
            ("ri jinvi lo du'u lo patfu cu curmi", "He thinks Dad agreed to give him money.", True),
            ("lo ri kelci girzu cu fliba", "His team is losing.", False),
            ("ri pu facki lo du'u ri ponse lo jdini", "He found out he had money.", False)]),
       2: ch(".i lo patfu be la .djunior. cu mo", "Junior's dad...", [
            ("pu bevri ri lo barda zarci", "took Junior to the mall", False),
            ("noroi catlu lo kelci", "never watches sports", False),
            ("pu mutce se cinri lo nu catlu lo boltipterjvi .i se ki'u bo ri na tirna la .djunior.", "was so into the game he didn't listen", True)])},
    D=[("line 3", ".i .e'u .e'u .e'u", "cheering \"Come on!\" as repeated .e'u (suggestion); .e'a or ko-imperatives are alternatives")],
)


# --- second pass (after the first validator run) ---------------------------
# A {(question, answer): jbo} replaces one answer of a choice question.
FIXES["batch2-01"]["A"] = {(2, 2): "lo bevri cu gasnu lo nu la .alinas. cu cliva lo gusta"}
FIXES["batch2-09"].setdefault("L", {})[16] = ".i .ei do kansa mi vi lo zdani .i do jai bandu mi"
FIXES["batch2-18"].setdefault("L", {})[6] = ".i .ei mi jukpa lo mapti vanci sanmi mu'i lo nu mi'a salci pa jbedetri"
FIXES["batch2-12"].setdefault("L", {})[13] = ".i mi ponse re pikta be fi lo nu zgike ca lo cabycte"
FIXES["batch2-13"].setdefault("Q", {})[0] = {"type": "fill", "line_index": 4, "speaker": "Renata", "answer": "burgere", "jbo": "", "en": ""}
FIXES["batch2-32"].setdefault("Q", {})[0] = {"type": "fill", "line_index": 3, "speaker": "Fabio", "answer": "nunsla", "jbo": "", "en": ""}
