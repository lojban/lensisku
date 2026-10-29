"""Source for course/course.json: a 30-lesson progressive Lojban course.

Follows lojban.pw .../lojban-through-dialogues/new-dialogue-plan.md (Pimsleur-style
arc with Lifri, Mentu and Linto).  Each lesson = new words + grammar note +
isolated phrases + one short dialogue.  Built by scripts/build_course.py.

Lesson tuple fields: (title_en, title_jbo, new_words, grammar, phrases, lines[, doubtful])
  phrases: [(jbo, en)]       lines: [(speaker, jbo, en)]   speaker '>' = narrator
  doubtful: [(where, candidate, note)]   where = 'phrase N' / 'line N'
"""

SPEAKERS = {"Lifri": "F", "Mentu": "M", "Linto": "F", "Newcomer": "M"}
NAMES = {"Lifri": ".lifris.", "Mentu": ".mentus.", "Linto": ".lintas."}

BLOCKS = [(1, 6, "The Meetup"), (7, 13, "Daily Life"), (14, 21, "Getting to Know Each Other"),
          (22, 29, "Deeper Conversation"), (30, 30, "Capstone")]

# Where the plan's Lojban was changed (reported in course.json and the review notes).
PLAN_DEVIATIONS = [
    "Names are cmevla .lifris. / .mentus. / .lintas. (lifri, mentu, linto are gismu; a cmevla must end in a consonant).",
    "L6: clock formula is `ti'u li N` (the plan's `li cacra bu N` is not needed and `cacra bu` is not a word).",
    "L12: `se jdini` is not a price expression; prices use `se jdima lo meryru'u be li N` (jdima x1 price of x2).",
    "L15: `se panzi re` is ungrammatical; `mi se panzi re da` = I have two children.",
    "L20: `taske` means thirsty, not tired; tired is `tatpi` (both taught). `se cinmo` -> `cinmo` (x1 feels emotion x2).",
    "L23/26: `taksi` is only an experimental gismu, so the trip is `fu lo karce` (by car); kept as a doubtful note.",
    "L24: `sepli` means apart/separate, not distance; distance uses `ki'otre` (x1 is x2 km long) and `darno`.",
    "L25: `mupli` dropped (no natural beginner sentence); comparison uses `zmadu`/`mleca` with `lo ka`.",
    "L22: `kanpe` (expect) is only an experimental gismu, so it was dropped.",
    "L18: `lerci` x1 is an event, so \"It's late\" is the observative `.i lerci`.",
]

L = []

L.append(("First Meeting", "lo pa moi nu penmi", ["coi", "co'o", "mi'e", "la", "cliva", "do"],
    "One word is already a full sentence: `coi` (hello), `co'o` (goodbye). `coi do` greets you; `coi .lifris.` greets Lifri by name. `mi'e la .mentus.` = \"I'm Mentu\".",
    [("coi", "Hello!"), ("co'o", "Goodbye!"), ("coi do", "Hello (to you)!"), ("mi'e la .mentus.", "I'm Mentu."),
     ("coi .lifris.", "Hi, Lifri!"), ("mi cliva", "I'm leaving."), ("do cliva", "You're leaving.")],
    [("Lifri", "coi do", "Hello!"),
     ("Mentu", "coi .i mi'e la .mentus.", "Hi! I'm Mentu."),
     ("Lifri", "mi'e la .lifris.", "I'm Lifri."),
     (">", ".i la .lintas. cu klama", "Linto arrives."),
     ("Linto", "coi .lifris. coi .mentus.", "Hi Lifri! Hi Mentu!"),
     ("Lifri", "coi .lintas.", "Hi, Linto!"),
     ("Mentu", ".i mi cliva .i co'o", "I'm leaving. Bye!"),
     ("Linto", "co'o .mentus.", "Bye, Mentu!")]))

L.append(("Do You Speak Lojban?", "xu do se bangu la .lojban.", ["xu", "se bangu", "go'i", "na", "milxe", "mi"],
    "`xu` turns a sentence into a yes/no question. Answer `go'i` (yes, it's so) or `na go'i` (no). `milxe` softens: `mi milxe se bangu` = I speak a little.",
    [("xu do se bangu la .lojban.", "Do you speak Lojban?"), ("go'i", "Yes."), ("na go'i", "No."),
     ("mi milxe se bangu la .lojban.", "I speak Lojban a little."), ("mi na se bangu la .lojban.", "I don't speak Lojban."),
     ("xu do se bangu lo glico bangu", "Do you speak English?")],
    [("Lifri", "coi .mentus. .i xu do se bangu la .lojban.", "Hi Mentu! Do you speak Lojban?"),
     ("Mentu", "go'i .i mi milxe se bangu la .lojban.", "Yes. I speak Lojban a little."),
     ("Linto", ".i xu do se bangu lo glico bangu", "Do you speak English?"),
     ("Mentu", "go'i", "Yes."),
     ("Linto", ".i mi na se bangu lo glico bangu", "I don't speak English."),
     ("Lifri", ".i ku'i do se bangu la .lojban.", "But you speak Lojban."),
     ("Linto", "go'i", "Yes!")]))

L.append(("What's Your Name? What Do You Like?", "ma cmene do", ["ma", "mo", "cmene", "zo", "nelci"],
    "`ma` asks for a missing noun, `mo` for a missing predicate. `zo` quotes one word: `zo .mentus. cmene mi` = \"Mentu\" is my name. `do mo` = how are you?",
    [("ma cmene do", "What's your name?"), ("zo .mentus. cmene mi", "My name is Mentu."), ("do nelci ma", "What do you like?"),
     ("mi nelci la .lojban.", "I like Lojban."), ("mi nelci lo ckafi", "I like coffee."), ("do mo", "How are you?")],
    [("Linto", "coi .i ma cmene do", "Hi! What's your name?"),
     ("Mentu", "zo .mentus. cmene mi .i ma cmene do", "My name is Mentu. What's your name?"),
     ("Linto", "mi'e la .lintas.", "I'm Linto."),
     ("Lifri", ".i do nelci ma", "What do you like?"),
     ("Linto", ".i mi nelci lo ckafi", "I like coffee."),
     ("Mentu", ".i mi nelci la .lojban.", "I like Lojban!"),
     ("Lifri", ".ui", "Great!")]))

L.append(("Thank You — Wrapping Up the Meetup", "ki'e", ["ki'e", "je'e", ".ui", ".u'u", "pei", ".e'u"],
    "Attitudinals are whole sentences or go before one. `pei` asks how the other person feels: `.ui pei` = are you happy? `.e'u` suggests: `.e'u mi'o ba penmi` = let's meet again.",
    [("ki'e", "Thanks!"), ("je'e", "You're welcome."), (".ui", "Yay!"), (".u'u", "Sorry."),
     (".ui pei", "Are you happy?"), (".e'u mi'o ba penmi", "Let's meet again.")],
    [("Mentu", "ki'e .lifris.", "Thank you, Lifri!"),
     ("Lifri", "je'e", "You're welcome."),
     ("Linto", ".ui .i mi nelci la .lojban.", "Yay! I like Lojban!"),
     ("Mentu", ".u'u doi .lintas.", "Sorry, Linto!"),
     ("Linto", "je'e", "It's OK."),
     ("Lifri", ".ui .i .ui pei", "I'm happy! And you?"),
     ("Mentu", ".ui", "Happy!"),
     ("Linto", ".e'u mi'o ba penmi", "Let's meet again!"),
     ("Lifri", ".ie .i co'o", "Yes! Bye!")],
    [("phrase 1", "je'e", "je'e (\"roger\") as the answer to ki'e follows common usage; CLL gives no dedicated \"you're welcome\""),
     ("line 1", "je'e", "same je'e usage")]))

L.append(("Let's Meet at the Park", "mi klama le panka", ["zvati", "klama", "le", "zdani", "panka"],
    "`le` = the (one I have in mind). `klama`: x1 goes to x2 from x3. `ma nuzba` = what's new?",
    [("ma nuzba", "What's new?"), ("do zvati ma", "Where are you?"), ("mi zvati le zdani", "I'm at home."),
     ("do klama ma", "Where are you going?"), ("mi klama le panka", "I'm going to the park.")],
    [("Lifri", "coi .mentus. .i ma nuzba", "Hi Mentu! What's new?"),
     ("Mentu", "no da", "Nothing."),
     ("Lifri", ".i do zvati ma", "Where are you?"),
     ("Mentu", ".i mi zvati le zdani .i do zvati ma", "I'm at home. Where are you?"),
     ("Lifri", ".i mi zvati le panka", "I'm at the park."),
     ("Mentu", ".ui .i mi klama le panka", "Great! I'm coming to the park."),
     ("Lifri", ".ui", "Great!")]))

L.append(("What Time? Let's Say Seven", "ti'u li ze", ["pa", "re", "ci", "vo", "mu", "xa", "ze", "bi", "so", "no", "tcika", "ti'u", "li"],
    "Digits chain left to right: `pano` = 10, `reno` = 20. `li` makes a number a noun. `ti'u li xa` = at six o'clock. `ma tcika` = what time is it?",
    [("li pa", "one"), ("li re", "two"), ("li ci", "three"), ("li ze", "seven"), ("li pano", "ten"),
     ("ma tcika", "What time is it?"), ("ti'u li xa", "(It's) six o'clock."), ("mi'o penmi ti'u li ze", "We meet at seven.")],
    [("Mentu", ".i ma tcika", "What time is it?"),
     ("Lifri", "ti'u li xa", "It's six."),
     ("Mentu", ".i xu mi'o penmi ti'u li ze", "Shall we meet at seven?"),
     ("Lifri", "go'i .i ti'u li ze", "Yes, at seven."),
     ("Mentu", ".i mi klama le panka ti'u li ze", "I'm going to the park at seven."),
     ("Lifri", ".ui .i co'o", "Great! Bye!")]))

L.append(("Good Morning", "cerni coi", ["cerni", "vanci", "nicte", "donri", "gleki", "ca"],
    "`ca le cerni` = in the morning; also `ca le vanci` (evening), `ca le nicte` (night). `cerni coi` / `donri coi` = good morning / good day.",
    [("cerni coi", "Good morning!"), ("donri coi", "Good day!"), ("mi gleki", "I'm happy."),
     ("ca le cerni mi zvati le zdani", "In the morning I'm at home."), ("ca le vanci mi klama le panka", "In the evening I go to the park."),
     ("ca le nicte mi zvati le zdani", "At night I'm at home.")],
    [("Mentu", "cerni coi .lifris.", "Good morning, Lifri!"),
     ("Lifri", "cerni coi .i do mo", "Good morning! How are you?"),
     ("Mentu", ".i mi gleki", "I'm happy."),
     ("Linto", "donri coi", "Good day!"),
     ("Lifri", ".i ca le cerni mi zvati le zdani", "In the morning I'm at home."),
     ("Lifri", ".i ca le vanci mi klama le panka", "In the evening I go to the park."),
     ("Linto", ".ui", "Great!")]))

L.append(("Speak Slowly! I Don't Understand", "ko masno tavla", ["sutra", "masno", "ko", "djuno", "jimpe", "preti"],
    "`ko` makes a command: `ko masno tavla` = speak slowly! `mi na jimpe` = I don't understand; `mi na djuno` = I don't know.",
    [("ko masno tavla", "Speak slowly!"), ("mi na jimpe", "I don't understand."), ("do sutra tavla", "You speak fast."),
     ("mi na djuno", "I don't know."), ("xu do jimpe", "Do you understand?"), ("mi ponse pa preti", "I have a question.")],
    [("Lifri", ".i do mo .i ma nuzba .i xu do zvati le panka ca le cerni", "How are you? What's new? Are you at the park in the mornings?"),
     ("Mentu", ".u'u .i mi na jimpe .i ko masno tavla", "Sorry! I don't understand. Speak slowly!"),
     ("Linto", ".i do sutra tavla", "You speak fast."),
     ("Lifri", ".u'u .i xu do jimpe", "Sorry. Do you understand now?"),
     ("Mentu", "go'i .i mi jimpe .i ki'e", "Yes, I understand. Thanks!"),
     ("Linto", ".i mi ponse pa preti .i ma tcika", "I have a question. What time is it?"),
     ("Mentu", ".i mi na djuno", "I don't know."),
     ("Lifri", "ti'u li ze", "Seven o'clock.")]))

L.append(("Yesterday and Tomorrow", "ca le prulamdei", ["pu", "ba", "ku'i", "cu", "cabdei", "bavlamdei", "prulamdei"],
    "`pu` = past, `ba` = future, before the predicate. `cu` separates a long subject from the predicate. `.i ku'i` = but. `cabdei` today, `prulamdei` yesterday, `bavlamdei` tomorrow.",
    [("xu do pu klama le panka", "Did you go to the park?"), ("go'i .i mi pu klama", "Yes, I went."),
     ("ca le bavlamdei mi ba klama le zdani", "Tomorrow I'll go home."), (".i ku'i ca le cabdei mi zvati ti", "But today I'm here."),
     ("le panka cu melbi", "The park is beautiful.")],
    [("Lifri", ".i xu do pu klama le panka ca le prulamdei", "Did you go to the park yesterday?"),
     ("Mentu", "go'i .i mi pu klama", "Yes, I went."),
     ("Linto", ".i ca le bavlamdei mi ba klama le zdani", "Tomorrow I'll go home."),
     ("Linto", ".i ku'i ca le cabdei mi zvati ti", "But today I'm here."),
     ("Lifri", ".i le panka cu melbi", "The park is beautiful."),
     ("Mentu", "go'i", "Yes!")]))

L.append(("I Want to Eat", "mi djica lo nu citka", [".au", ".ai", "citka", "pinxe", "lo", "djacu", "nanba"],
    "`.au` = I wish; `.ai` = I'm going to (intent). `lo` = some/a (any), `le` = the specific one.",
    [(".au mi citka", "I want to eat."), (".au mi pinxe lo djacu", "I'd like to drink water."),
     (".ai mi citka lo nanba", "I'm going to eat bread."), (".e'u mi'o klama lo gusta", "Let's go to a restaurant."),
     ("mi ji'a citka", "I eat too.")],
    [("Mentu", ".au mi citka", "I want to eat!"),
     ("Linto", ".au mi pinxe lo djacu", "I want to drink water."),
     ("Lifri", ".au mi ji'a citka", "I want to eat too."),
     ("Mentu", ".e'u mi'o klama lo gusta", "Let's go to a restaurant!"),
     ("Linto", ".ie", "Yes!"),
     ("Mentu", ".ai mi citka lo nanba", "I'm going to eat bread.")]))

L.append(("Ordering at the Café", "vi lo gusta", ["stasu", "cidja", "jisra", "titnanba", "gusta"],
    "`pei` again: after an order it means \"and you?\". Chain orders: `.ai mi citka lo stasu .i .ai mi pinxe lo jisra`.",
    [(".ai mi citka lo stasu", "I'm going to have soup."), (".ai mi pinxe lo jisra", "I'll have juice."),
     ("xu do nelci lo titnanba", "Do you like cookies?"), ("le cidja cu kukte", "The food is tasty."), ("lo gusta", "a restaurant")],
    [(">", ".i la .lifris. .e la .mentus. .e la .lintas. cu zvati lo gusta", "Lifri, Mentu and Linto are at a café."),
     ("Lifri", ".ai mi citka lo stasu .i .ai pei", "I'm going to have soup. And you?"),
     ("Mentu", ".ai mi pinxe lo jisra", "I'll have juice."),
     ("Lifri", ".i xu do nelci lo titnanba", "Do you like cookies?"),
     ("Linto", "go'i .i .ai mi citka lo titnanba .e lo nanba", "Yes! I'll have cookies and bread."),
     ("Mentu", ".i le cidja cu kukte", "The food is tasty!")]))

L.append(("How Much? Is It Expensive?", "xu ri kargu", ["kargu", "tolkargu", "meryru'u", "se jdima", "pleji", "te vecnu"],
    "`tol-` reverses: `kargu` expensive -> `tolkargu` cheap. Prices: `se jdima lo meryru'u be li N` = costs N dollars. `te vecnu` = buy, `pleji` = pay.",
    [("xu ri kargu", "Is it expensive?"), ("lo nanba cu se jdima lo meryru'u be li paci", "The bread costs 13 dollars."),
     ("ti tolkargu", "This is cheap."), ("xu do ponse lo jdini", "Do you have money?"), ("mi ba pleji", "I'll pay."),
     ("mi te vecnu lo stasu", "I buy soup.")],
    [("Mentu", ".i xu lo stasu cu kargu", "Is the soup expensive?"),
     ("Lifri", ".i lo nanba cu se jdima lo meryru'u be li paci", "The bread costs 13 dollars."),
     ("Linto", ".ue .i ri kargu", "Wow, that's expensive!"),
     ("Lifri", ".i lo stasu cu se jdima lo meryru'u be li reno", "The soup costs 20."),
     ("Mentu", ".i xu do ponse lo jdini", "Do you have money?"),
     ("Linto", ".i mi ba pleji", "I'll pay."),
     ("Mentu", "ki'e", "Thanks!")]))

L.append(("Buying Things", "lo zarci", ["zarci", ".e", "mutce", "drata"],
    "`.e` joins nouns: `lo titnanba .e lo jisra` = cookies and juice. `mutce` intensifies: `mutce kargu` = very expensive.",
    [("mi klama lo zarci", "I'm going to a shop."), ("mi ba te vecnu lo titnanba .e lo jisra", "I'll buy cookies and juice."),
     ("xu lo jisra cu kargu", "Is the juice expensive?"), ("ri na kargu .i ri tolkargu", "It's not expensive, it's cheap."),
     ("lo cidja cu mutce kargu", "Food is very expensive.")],
    [("Mentu", ".i mi klama lo zarci", "I'm going to a shop."),
     ("Mentu", ".i mi ba te vecnu lo titnanba .e lo jisra", "I'll buy cookies and juice."),
     ("Linto", ".i xu lo jisra cu kargu", "Is the juice expensive?"),
     ("Mentu", "na go'i .i ri tolkargu", "No, it's cheap."),
     ("Linto", ".i ku'i lo cidja cu mutce kargu", "But the food is very expensive."),
     ("Mentu", "go'i .i mi ba te vecnu lo drata jisra", "Yes. I'll buy a different juice.")]))

L.append(("Planning the Week", "ca ro jeftu", ["birti", "jeftu", "djedi", "ro", "detma'i"],
    "`ro` = every: `ca ro jeftu` = every week, `ca ro detma'i` = every month. `lo jeftu be li pa` = a one-week period. `mi birti` = I'm sure.",
    [("ca le cabdei mi zvati le zdani", "Today I'm at home."), ("ca le bavlamdei mi ba klama le panka", "Tomorrow I'll go to the park."),
     ("mi birti", "I'm sure."), ("ca ro jeftu mi'o penmi", "Every week we meet."), ("lo jeftu be li pa", "a one-week period"),
     ("ca ro detma'i", "every month")],
    [("Lifri", ".i xu mi'o penmi ca ro jeftu", "Shall we meet every week?"),
     ("Mentu", "go'i .i mi birti", "Yes, I'm sure."),
     ("Linto", ".i ca le cabdei mi zvati le zdani", "Today I'm at home."),
     ("Linto", ".i ku'i ca le bavlamdei mi ba klama le panka", "But tomorrow I'll go to the park."),
     ("Lifri", ".i xu do birti", "Are you sure?"),
     ("Linto", "go'i .i mi birti", "Yes, I'm sure."),
     ("Mentu", ".i ca ro detma'i mi'o klama lo gusta", "And every month we go to a restaurant.")]))

L.append(("My Family", "le lanzu be mi", ["speni", "se panzi", "lanzu", "be", "xo"],
    "`be` attaches a place inside a noun: `le speni be mi` = my spouse. `xo` asks for a number: `do se panzi xo da` = how many children do you have?",
    [("xu do speni", "Are you married?"), ("le speni be mi cu gleki", "My spouse is happy."),
     ("xu do se panzi", "Do you have children?"), ("do se panzi xo da", "How many children do you have?"),
     ("mi se panzi re da", "I have two children."), ("le lanzu be mi cu gleki", "My family is happy.")],
    [("Linto", ".i xu do speni", "Are you married?"),
     ("Mentu", "go'i .i le speni be mi cu gleki", "Yes. My spouse is happy."),
     ("Linto", ".i xu do se panzi", "Do you have children?"),
     ("Mentu", "go'i", "Yes."),
     ("Linto", ".i do se panzi xo da", "How many?"),
     ("Mentu", ".i mi se panzi re da", "Two."),
     ("Mentu", ".i le lanzu be mi cu gleki", "My family is happy.")]))

L.append(("Son, Daughter, Siblings", "le bersa be mi", ["bersa", "tixnu", "bruna", "mensi", "nanla", "nixli"],
    "`le bersa be mi` = my son (x2 of bersa is the parent). `se bruna` = to have a brother. `nanla` / `nixli` = boy / girl.",
    [("le bersa be mi cu gleki", "My son is happy."), ("le tixnu be mi cu zvati le zdani", "My daughter is at home."),
     ("mi se panzi pa nanla .e pa nixli", "I have a boy and a girl."), ("le tixnu be do cu tavla mi", "Your daughter talks to me."),
     ("xu do se bruna", "Do you have a brother?"), ("mi se mensi", "I have a sister.")],
    [("Mentu", ".i le bersa be mi cu gleki", "My son is happy."),
     ("Mentu", ".i le tixnu be mi cu zvati le zdani", "My daughter is at home."),
     ("Linto", ".i xu do se bruna", "Do you have a brother?"),
     ("Mentu", "na go'i .i ku'i mi se mensi", "No, but I have a sister."),
     ("Lifri", ".i le tixnu be do pu tavla mi", "Your daughter talked to me."),
     ("Mentu", ".ui", "Oh, nice!")]))

L.append(("Where Do You Live?", "do xabju ma", ["xabju", "tcadu", "barda", "cmalu", "jibni", "darno"],
    "`barda` / `cmalu` = big / small; `jibni` / `darno` = near / far. `le zdani be mi` = my home.",
    [("do xabju ma", "Where do you live?"), ("mi xabju lo barda tcadu", "I live in a big city."),
     ("le zdani be mi cu cmalu", "My home is small."), ("xu le tcadu cu darno", "Is the city far?"),
     ("le panka cu jibni", "The park is near.")],
    [("Lifri", ".i do xabju ma", "Where do you live?"),
     ("Mentu", ".i mi xabju lo barda tcadu", "I live in a big city."),
     ("Mentu", ".i ku'i le zdani be mi cu cmalu", "But my home is small."),
     ("Lifri", ".i xu le tcadu cu darno", "Is the city far?"),
     ("Mentu", "na go'i .i ri jibni", "No, it's near."),
     ("Linto", ".i xu le panzi be do cu xabju le zdani be do", "Do your children live with you?"),
     ("Mentu", "go'i", "Yes.")]))

L.append(("I Need to Work", "mi nitcu lo nu gunka", ["sipna", "gunka", "nitcu", "jibri", "lerci", "lo nu"],
    "`lo nu ...` turns a sentence into an event: `mi nitcu lo nu mi gunka` = I need to work. `lo jibri be do cu mo` = what's your job?",
    [(".i lerci", "It's late."), ("mi ba sipna", "I'm going to sleep."), ("mi nitcu lo nu mi gunka", "I need to work."),
     ("lo jibri be do cu mo", "What is your job?"), ("mi gunka ca le cerni", "I work in the morning."),
     ("mi nitcu lo nu sipna", "I need to sleep.")],
    [("Mentu", ".i lerci .i mi ba sipna", "It's late. I'm going to sleep."),
     ("Linto", ".i xu ca", "Now?"),
     ("Mentu", "go'i .i ca le bavlamdei mi nitcu lo nu mi gunka", "Yes. Tomorrow I need to work."),
     ("Linto", ".i lo jibri be do cu mo", "What's your job?"),
     ("Mentu", ".i mi ctuca", "I'm a teacher."),
     ("Mentu", ".i mi gunka ca le cerni .i mi nitcu lo nu sipna", "I work in the morning. I need to sleep."),
     ("Linto", "co'o .mentus.", "Bye, Mentu!")]))

L.append(("Travel and Countries", "lo nu litru", ["litru", "gugde", "fi'i", "bangu", "drata"],
    "Names of places take `la`: `la .paris.`, `la .fransis.`. `lo drata gugde` = another country. `fi'i` = welcome!",
    [("fi'i", "Welcome!"), ("xu do pu litru", "Have you travelled?"), ("mi pu klama lo drata gugde", "I went to another country."),
     ("vi ta do se bangu ma", "What language do you speak there?"), ("mi milxe se bangu lo fraso bangu", "I speak a little French."),
     ("xu la .fransis. cu darno", "Is France far?")],
    [("Lifri", "fi'i .lintas.", "Welcome back, Linto!"),
     ("Mentu", ".i xu do pu litru", "Have you been travelling?"),
     ("Linto", "go'i .i mi pu klama lo drata gugde", "Yes, I went to another country."),
     ("Linto", ".i mi pu zvati la .paris.", "I was in Paris."),
     ("Mentu", ".i vi ta do se bangu ma", "What language do you speak there?"),
     ("Linto", ".i mi milxe se bangu lo fraso bangu", "I speak a little French."),
     ("Lifri", ".i xu la .paris. cu darno", "Is Paris far?"),
     ("Linto", "go'i", "Yes.")]))

L.append(("I'm Tired — How Do You Feel?", "mi tatpi", ["tatpi", "taske", "kufra", "zabna", "pluka", "cinmo"],
    "`tatpi` = tired, `taske` = thirsty. `lo nu X cu pluka mi` = X pleases me. `mi cinmo lo ka gleki` = I feel happy.",
    [("mi tatpi", "I'm tired."), ("mi taske", "I'm thirsty."), ("lo nu cadzu pu pluka mi", "The walk was pleasant."),
     ("mi kufra", "I feel comfortable."), ("lo bavlamdei ba zabna", "Tomorrow will be nice."), ("mi cinmo lo ka gleki", "I feel happy.")],
    [("Lifri", ".i do mo", "How are you?"),
     ("Mentu", ".i mi tatpi", "I'm tired."),
     ("Linto", ".i mi taske .i .au mi pinxe lo djacu", "I'm thirsty. I want to drink water."),
     ("Lifri", ".i ku'i lo nu cadzu pu pluka mi", "But the walk was pleasant."),
     ("Mentu", "go'i .i mi ca kufra", "Yes. Now I feel comfortable."),
     ("Linto", ".i lo bavlamdei ba zabna", "Tomorrow will be nice."),
     ("Lifri", ".e'u mi'o tavla ca le vanci", "Let's talk in the evening.")]))

L.append(("Let's Go Together", "mi'o kansa klama", ["casnu", "jo'u", "prami", "tadni", "pendo"],
    "`jo'u` joins people acting together: `mi jo'u le speni be mi` = my spouse and I. `mi prami do` = I love you; `do pendo mi` = you are my friend.",
    [(".e'u mi'o casnu", "Let's discuss."), ("xu mi'o kansa klama", "Shall we go together?"),
     ("mi jo'u le speni be mi ba klama", "My spouse and I will go."), ("mi tadni la .lojban.", "I study Lojban."),
     ("mi prami do", "I love you."), ("do pendo mi", "You're my friend.")],
    [("Lifri", ".e'u mi'o casnu lo nu litru", "Let's discuss the trip."),
     ("Lifri", ".i xu mi'o kansa klama le mi tcadu", "Shall we go to my city together?"),
     ("Mentu", "go'i .i mi jo'u le speni be mi ba klama", "Yes! My spouse and I will go."),
     ("Linto", ".i mi ji'a ba klama", "I'll go too."),
     ("Linto", ".i mi tadni la .lojban. .i mi mutce nelci ri", "I study Lojban. I really like it!"),
     ("Mentu", "doi pendo .i mi prami do", "Friends, I love you!"),
     ("Lifri", ".e'u mi'o klama ca le vanci", "Let's go in the evening.")]))

L.append(("How Long Will You Stay?", "ze'a ma", ["ze'i", "ze'a", "ze'u", "stali", "djedi"],
    "Duration: `ze'i` short, `ze'a` medium, `ze'u` long. `ze'a lo jeftu be li pa` = for a week. `lo djedi be li ci` = three days.",
    [("mi ba stali ze'a lo jeftu be li pa", "I'll stay for a week."), ("mi ba ze'i gunka", "I'll work for a short while."),
     ("mi ba stali ze'a lo djedi be li ci", "I'll stay for three days."), ("xu do ba ze'u stali", "Will you stay long?"),
     ("na go'i .i mi ba ze'i stali", "No, I'll stay a short time.")],
    [("Mentu", ".i mi'o ba stali ze'a ma", "How long will we stay?"),
     ("Lifri", ".i mi'o ba stali ze'a lo jeftu be li pa", "We'll stay for a week."),
     ("Linto", ".i ku'i mi ba stali ze'a lo djedi be li ci", "But I'll only stay three days."),
     ("Linto", ".i mi nitcu lo nu gunka", "I need to work."),
     ("Mentu", ".i xu do ba ze'u stali", "So you won't stay long?"),
     ("Linto", "na go'i .i mi ba ze'i stali", "No, I'll stay a short time.")]))

L.append(("Visiting — Welcome!", "fi'i", ["vitke", "fu", "karce", "ti", "ta", "tu", ".ue"],
    "`fu` marks the 5th place, for `klama` the vehicle: `klama fu lo karce` = go by car. `ti` / `ta` / `tu` = this here / that there / that over yonder. `.ue` = surprise!",
    [("fi'i do", "Welcome!"), ("ko nerkla", "Come in!"), ("lo nu litru pu mo", "How was the trip?"),
     ("mi'o klama fu lo karce", "We came by car."), ("ti zdani mi", "This is my home."), (".ue", "Oh, what a surprise!"),
     ("mi vitke do", "I'm your guest.")],
    [("Lifri", "fi'i .i ko nerkla", "Welcome! Come in!"),
     ("Mentu", "ki'e .i ti melbi zdani", "Thanks! This is a beautiful home."),
     ("Lifri", ".i lo nu litru pu mo", "How was the trip?"),
     ("Linto", ".i mi'o klama fu lo karce", "We came by car."),
     ("Lifri", ".i xu lo klaji pu clani", "Was the road long?"),
     ("Mentu", "go'i .i lo klaji cu ki'otre li muno", "Yes, the road is fifty kilometres."),
     ("Lifri", ".ue", "Wow!")],
    [("phrase 3", "mi'o klama fu lo taksi", "the plan has `taksi`, but taksi is only an experimental gismu; the course uses lo karce (car)")]))

L.append(("How Far? How Many?", "xo prenu", ["xo", "ki'otre", "ro", "so'i"],
    "`xo` again: `xo prenu cu zvati ti` = how many people are here? `lo klaji cu ki'otre li muno` = the road is 50 km long. `so'i` = many.",
    [("xo prenu cu zvati ti", "How many people are here?"), ("xu le panka cu darno", "Is the park far?"),
     ("lo klaji be le tcadu cu ki'otre li muno", "The road to the city is 50 km long."), ("so'i zarci cu zvati ti", "There are many shops here."),
     ("ro zarci cu cmalu", "Every shop is small."), ("do se panzi xo da", "How many children do you have?")],
    [("Mentu", ".i xo prenu cu xabju le tcadu", "How many people live in the city?"),
     ("Lifri", "so'i prenu", "Many people."),
     ("Linto", ".i xu le panka cu darno", "Is the park far?"),
     ("Lifri", "na go'i .i ri jibni", "No, it's near."),
     ("Mentu", ".i lo klaji be le mi tcadu cu ki'otre li muno", "The road to my city is fifty kilometres."),
     ("Lifri", "doi .mentus. .i do se panzi xo da", "Mentu, how many children do you have again?"),
     ("Mentu", ".i mi se panzi re da", "Two!")]))

L.append(("Opinions and Preferences", "lo se jinvi", ["jinvi", "simlu", "zmadu", "mleca", "lo du'u"],
    "`mi jinvi lo du'u X` = I think that X. `ta simlu lo ka zabna` = that seems nice. `ti zmadu ta lo ka kukte` = this is tastier than that; `mleca` = less.",
    [("do jinvi ma", "What do you think?"), ("mi jinvi lo du'u ri zabna", "I think it's nice."),
     ("ta simlu lo ka zabna", "That seems nice."), ("ti zmadu ta lo ka kukte", "This is tastier than that."),
     ("ti mleca ta lo ka kargu", "This is cheaper than that.")],
    [(">", ".i le ci pendo cu citka lo vanci sanmi vi le zdani be la .lifris.", "The three friends have dinner at Lifri's home."),
     ("Lifri", ".i do jinvi ma", "What do you think?"),
     ("Mentu", ".i mi jinvi lo du'u le stasu cu zabna", "I think the soup is great."),
     ("Linto", ".i le nanba ji'a simlu lo ka kukte", "The bread also seems tasty."),
     ("Lifri", ".i le stasu ji le nanba cu zmadu fi lo ka kukte", "Which is tastier, the soup or the bread?"),
     ("Mentu", ".i le stasu", "The soup."),
     ("Linto", ".i mi jinvi lo du'u la .lojban. cu zmadu ro drata bangu lo ka xamgu", "In my opinion, Lojban is the best language."),
     ("Lifri", ".ie", "Agreed!")]))

L.append(("Coming Back", "mi di'a klama", ["di'a", "pe", "ba le nu ... kei", "coi ro do", "co'o ro do"],
    "`ba le nu vitke kei` = after the visit (`kei` closes the `nu`). `pe` = loose possession: `le karce pe do` = your car. `di'a` = resume: `mi ba di'a gunka` = I'll get back to work.",
    [("coi ro do", "Hello, everyone!"), ("ba le nu vitke kei mi ba klama le zdani", "After the visit I'll go home."),
     ("xu do ba klama fu le karce pe do", "Will you go in your car?"), ("mi ba di'a gunka", "I'll get back to work."),
     ("ca le nu do xruti kei ko fonta'a mi", "When you get back, call me."), ("co'o ro do", "Goodbye, everyone!")],
    [("Mentu", "coi ro do", "Good morning, everyone!"),
     ("Lifri", ".i xu do ba klama fu le karce pe do", "Will you go in your car?"),
     ("Mentu", "go'i .i ba le nu vitke kei mi ba klama le zdani", "Yes. After the visit I'll go home."),
     ("Linto", ".i mi ba di'a gunka", "I'll get back to work."),
     ("Lifri", ".i ca le nu do xruti kei ko fonta'a mi", "When you get back, call me!"),
     ("Mentu", ".i'e .i mi ba zi xruti ti", "OK! I'll come back here soon."),
     ("Linto", "co'o ro do", "Goodbye, everyone!")]))

L.append(("Paying Bills — Monthly Life", "ca ro detma'i", ["dikca", "kagni", "ca ro detma'i", "pleji"],
    "`ca ro detma'i` = every month. `pleji fi lo kagni` = pay the company (x3 of pleji is the recipient). `ro zdani` = every home.",
    [("xu lo dikca cu kargu", "Is electricity expensive?"), ("mi na djuno .i mi na pleji", "I don't know, I don't pay."),
     ("xu do pleji fi lo kagni", "Do you pay the company?"), ("ca ro detma'i mi pleji lo meryru'u be li reno", "Every month I pay 20 dollars."),
     ("vi ti lo dikca cu tolkargu", "Electricity is cheap here."), ("ro zdani cu nitcu tu'a lo dikca", "Every home needs electricity.")],
    [("Linto", ".i xu lo dikca cu kargu", "Is electricity expensive?"),
     ("Mentu", ".i mi na djuno .i le speni be mi cu pleji", "I don't know. My spouse pays."),
     ("Linto", ".i xu ri pleji fi lo kagni", "Does she pay the company?"),
     ("Mentu", "go'i .i ca ro detma'i ri pleji lo meryru'u be li reno", "Yes, every month she pays 20 dollars."),
     ("Linto", ".ue .i vi ti lo dikca cu tolkargu", "Wow, electricity is cheap here."),
     ("Mentu", ".i ku'i ro zdani cu nitcu tu'a lo dikca", "But every home needs electricity.")]))

L.append(("Because and Should", "ki'u ma", ["ki'u", ".ei", "co'a", "pu'o", "zi"],
    "`ki'u le nu X` = because X. `.ei` = I should. `co'a` = begin: `co'a sipna` = fall asleep. `pu'o nicte` = night is about to come. `ba zi` = soon.",
    [("mi klama le zdani ki'u le nu mi nitcu lo nu sipna", "I'm going home because I need to sleep."),
     (".ei mi sipna", "I should sleep."), ("mi ba zi co'a sipna", "I'll fall asleep soon."), ("pu'o nicte", "Night is coming."),
     ("lo nu tavla do kei pu pluka mi", "Talking to you was a pleasure.")],
    [("Mentu", ".u'u .i mi klama le zdani ki'u le nu mi nitcu lo nu sipna", "Sorry, I'm going home because I need to sleep."),
     ("Linto", ".i xu ca", "Already?"),
     ("Mentu", "go'i .i .ei mi sipna .i pu'o nicte", "Yes. I should sleep. Night is coming."),
     ("Lifri", ".i lo nu tavla do kei pu pluka mi", "Talking to you was a pleasure."),
     ("Mentu", ".i mi ji'a gleki", "I'm glad too."),
     ("Mentu", "co'o .i mi ba zi co'a sipna", "Goodbye! I'll be asleep soon.")]))

L.append(("Future Hopes", "mi pacna", ["pacna", "cumki", "tcini", "fenki", "bau"],
    "`mi pacna lo nu X` = I hope X happens. `lo nu X cu cumki` = X is possible. `bau la .lojban.` = in Lojban.",
    [("mi pacna lo nu mi sutra tavla bau la .lojban.", "I hope to speak Lojban fluently."),
     ("lo nu mi ba litru cu cumki", "Maybe I'll travel."), ("mi pacna lo nu ro do gleki", "I wish you all happiness."),
     ("lo tcini be mi cu xamgu", "My situation is good."), ("lo nu litru ca ro djedi cu fenki", "Travelling every day is crazy."),
     ("mi prami le vi bangu", "I love this language.")],
    [("Linto", ".i mi pacna lo nu mi sutra tavla bau la .lojban.", "I hope to speak Lojban fluently."),
     ("Lifri", ".i lo nu go'i cu cumki", "That's possible!"),
     ("Mentu", ".i lo nu mi ba litru cu cumki", "Maybe I'll travel."),
     ("Linto", ".i lo nu litru ca ro djedi cu fenki", "Travelling every day is crazy!"),
     ("Mentu", ".i mi pacna lo nu ro do gleki", "I wish you all happiness."),
     ("Lifri", ".e'u mi'o ba penmi", "Let's meet again."),
     ("Linto", ".i mi prami le vi bangu", "I love this language!")]))

L.append(("Full Spiral Review — One Year Later", "ba lo nanca be li pa", [],
    "Review of all 30 lessons. Mentu and Linto now speak confidently, and a newcomer says `coi`: the cycle begins again.",
    [("coi ro do .i ma nuzba", "Hi everyone! What's new?"), ("mi ca se bangu la .lojban.", "Now I speak Lojban."),
     ("ma cmene do", "What's your name?"), ("fi'i", "Welcome!")],
    [(">", ".i ba lo nanca be li pa le ci pendo cu penmi vi le panka", "One year later, the three friends meet at the park."),
     ("Lifri", "coi ro do .i ma nuzba", "Hi everyone! What's new?"),
     ("Mentu", ".ui .i mi ca se bangu la .lojban.", "I speak Lojban now!"),
     ("Linto", ".i mi pu di'a klama la .paris.", "I went to Paris again."),
     ("Lifri", ".i xu do ba stali ti ze'a lo jeftu be li re", "Will you stay here for two weeks?"),
     ("Linto", "na go'i .i mi nitcu lo nu gunka", "No, I need to work."),
     ("Mentu", ".i mi ca'o jinvi lo du'u la .lojban. cu zmadu ro drata bangu lo ka xamgu", "I still think Lojban is the best language."),
     (">", ".i pa cnino prenu cu klama", "A new person walks up."),
     ("Newcomer", "coi", "Hi..."),
     ("Lifri", "coi do .i mi'e la .lifris. .i ma cmene do", "Hello! I'm Lifri. What's your name?"),
     ("Mentu", "fi'i", "Welcome!")]))

assert len(L) == 30, len(L)

# Existing duostories-style stories linked as reading levels after a lesson.
READINGS = {7: ["01-coi-ca-lo-cerni"], 11: ["02-sakta-joi-silna", "03-lo-gusta"], 13: ["04-lo-taxfu", "05-lo-zarci"]}
READINGS_AFTER_COURSE = [f"batch2-{i:02d}" for i in range(1, 41)]
