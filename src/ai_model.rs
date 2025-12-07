use nalgebra::{DMatrix, DVector};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Простая нейронная сеть с поддержкой fp64 для высокоточного обучения
#[derive(Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub layers: Vec<Layer>,
    pub learning_rate: f64,
    pub vocab: HashMap<String, usize>,
    pub reverse_vocab: HashMap<usize, String>,
    pub embedding_dim: usize,
    pub hidden_dim: usize,
    pub context_length: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Layer {
    pub weights: Vec<Vec<f64>>,
    pub biases: Vec<f64>,
    pub activation: ActivationType,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ActivationType {
    ReLU,
    Tanh,
    Sigmoid,
    Softmax,
}

impl AIModel {
    pub fn new(embedding_dim: usize, hidden_dim: usize, context_length: usize) -> Self {
        let mut model = Self {
            layers: Vec::new(),
            learning_rate: 0.001,
            vocab: HashMap::new(),
            reverse_vocab: HashMap::new(),
            embedding_dim,
            hidden_dim,
            context_length,
        };
        
        // Инициализация базового словаря
        model.init_vocab();
        
        // Создание слоев нейронной сети
        model.init_layers();
        
        model
    }
    
    fn init_vocab(&mut self) {
        // МЕГА-РАСШИРЕННЫЙ русский и английский словарь (1000+ слов)
        let base_words = vec![
            // Приветствия и вежливость
            "привет", "здравствуй", "здравствуйте", "добрый", "день", "утро", "вечер", "ночь",
            "пока", "до", "свидания", "спасибо", "благодарю", "пожалуйста", "извините", "простите", "прости",
            "здорово", "приветствую", "салют", "хай", "алло", "доброго", "добро", "пожаловать",
            "hello", "hi", "hey", "goodbye", "bye", "thanks", "thank", "please", "sorry", "excuse", "pardon",
            "welcome", "greetings", "cheers", "howdy",
            
            // Местоимения и указательные
            "я", "ты", "он", "она", "оно", "мы", "вы", "они", "мой", "твой", "его", "её", "наш", "ваш", "их",
            "этот", "эта", "это", "эти", "тот", "та", "те", "весь", "все", "всё", "всех", "какой", "который", "каков",
            "такой", "такая", "такое", "такие", "сам", "сама", "само", "сами", "себя", "свой", "своя", "своё",
            "i", "you", "he", "she", "it", "we", "they", "my", "your", "his", "her", "its", "our", "their",
            "this", "that", "these", "those", "what", "which", "who", "whom", "whose", "where", "when", "why", "how",
            "myself", "yourself", "himself", "herself", "itself", "ourselves", "themselves",
            
            // Вопросительные и относительные
            "кто", "что", "где", "куда", "откуда", "когда", "почему", "зачем", "как", "сколько", "насколько",
            "который", "какой", "каким", "каком", "какая", "какую", "чей", "чья", "чьё", "чьи", "чего", "чему", "чем",
            "ли", "разве", "неужели", "что-то", "кто-то", "где-то", "когда-то", "как-то",
            
            // Глаголы (базовые и частотные)
            "быть", "есть", "был", "была", "было", "были", "буду", "будешь", "будет", "будем", "будете", "будут",
            "делать", "сделать", "делаю", "делает", "делают", "делал", "сделал", "сделаю",
            "иметь", "имею", "имеет", "имел", "знать", "знаю", "знает", "знал", "узнать",
            "мочь", "могу", "может", "можно", "нельзя", "мог", "смочь", "смогу",
            "хотеть", "хочу", "хочет", "хотел", "захотеть",
            "идти", "иду", "идёт", "идут", "шёл", "шла", "пойти", "прийти", "приду", "придёт",
            "говорить", "говорю", "говорит", "говорил", "сказать", "скажу", "сказал",
            "видеть", "вижу", "видит", "видел", "увидеть",
            "понимать", "понимаю", "понимает", "понял", "поняла", "понять",
            "думать", "думаю", "думает", "думал", "подумать",
            "работать", "работаю", "работает", "работал", "поработать",
            "любить", "люблю", "любит", "любил", "полюбить",
            "жить", "живу", "живёт", "жил", "прожить",
            "дать", "даю", "даёт", "дал", "дала", "дадут",
            "взять", "беру", "берёт", "взял", "возьму",
            "смотреть", "смотрю", "смотрит", "смотрел", "посмотреть",
            "читать", "читаю", "читает", "прочитать", "прочитал",
            "писать", "пишу", "пишет", "написать", "написал",
            "слышать", "слышу", "слышит", "услышать",
            "спрашивать", "спросить", "спрашиваю", "спросил",
            "отвечать", "ответить", "отвечаю", "ответил",
            "помогать", "помочь", "помогаю", "помог",
            "начинать", "начать", "начинаю", "начал",
            "кончать", "кончить", "закончить", "заканчивать",
            "продолжать", "продолжить", "продолжаю",
            "становиться", "стать", "становлюсь", "стал",
            "оставаться", "остаться", "остаюсь", "остался",
            "находить", "найти", "нахожу", "нашёл",
            "терять", "потерять", "теряю", "потерял",
            "искать", "ищу", "ищет", "искал",
            "получать", "получить", "получаю", "получил",
            "использовать", "используя", "использовал",
            "создавать", "создать", "создаю", "создал",
            "открывать", "открыть", "открываю", "открыл",
            "закрывать", "закрыть", "закрываю", "закрыл",
            "показывать", "показать", "показываю", "показал",
            "считать", "счесть", "считаю", "считал",
            "нести", "несу", "несёт", "нёс", "принести",
            "вести", "веду", "ведёт", "вёл", "привести",
            "играть", "играю", "играет", "играл", "сыграть",
            "учить", "учу", "учит", "учил", "выучить", "научить",
            "учиться", "учусь", "учится", "учился", "выучиться",
            "изучать", "изучаю", "изучает", "изучил",
            "понадобиться", "нужно", "надо", "требуется",
            
            "be", "is", "am", "are", "was", "were", "been", "being",
            "have", "has", "had", "having",
            "do", "does", "did", "done", "doing",
            "make", "makes", "made", "making",
            "get", "gets", "got", "gotten", "getting",
            "know", "knows", "knew", "known", "knowing",
            "think", "thinks", "thought", "thinking",
            "see", "sees", "saw", "seen", "seeing",
            "come", "comes", "came", "coming",
            "want", "wants", "wanted", "wanting",
            "use", "uses", "used", "using",
            "find", "finds", "found", "finding",
            "give", "gives", "gave", "given", "giving",
            "tell", "tells", "told", "telling",
            "work", "works", "worked", "working",
            "call", "calls", "called", "calling",
            "try", "tries", "tried", "trying",
            "ask", "asks", "asked", "asking",
            "need", "needs", "needed", "needing",
            "feel", "feels", "felt", "feeling",
            "become", "becomes", "became", "becoming",
            "leave", "leaves", "left", "leaving",
            "put", "puts", "putting",
            "mean", "means", "meant", "meaning",
            "keep", "keeps", "kept", "keeping",
            "let", "lets", "letting",
            "begin", "begins", "began", "begun", "beginning",
            "seem", "seems", "seemed", "seeming",
            "help", "helps", "helped", "helping",
            "talk", "talks", "talked", "talking",
            "turn", "turns", "turned", "turning",
            "start", "starts", "started", "starting",
            "show", "shows", "showed", "shown", "showing",
            "hear", "hears", "heard", "hearing",
            "play", "plays", "played", "playing",
            "run", "runs", "ran", "running",
            "move", "moves", "moved", "moving",
            "live", "lives", "lived", "living",
            "believe", "believes", "believed", "believing",
            "bring", "brings", "brought", "bringing",
            "happen", "happens", "happened", "happening",
            "write", "writes", "wrote", "written", "writing",
            "provide", "provides", "provided", "providing",
            "sit", "sits", "sat", "sitting",
            "stand", "stands", "stood", "standing",
            "lose", "loses", "lost", "losing",
            "pay", "pays", "paid", "paying",
            "meet", "meets", "met", "meeting",
            "include", "includes", "included", "including",
            "continue", "continues", "continued", "continuing",
            "set", "sets", "setting",
            "learn", "learns", "learned", "learnt", "learning",
            "change", "changes", "changed", "changing",
            "lead", "leads", "led", "leading",
            "understand", "understands", "understood", "understanding",
            "watch", "watches", "watched", "watching",
            "follow", "follows", "followed", "following",
            "stop", "stops", "stopped", "stopping",
            "create", "creates", "created", "creating",
            "speak", "speaks", "spoke", "spoken", "speaking",
            "read", "reads", "reading",
            "allow", "allows", "allowed", "allowing",
            "add", "adds", "added", "adding",
            "spend", "spends", "spent", "spending",
            "grow", "grows", "grew", "grown", "growing",
            "open", "opens", "opened", "opening",
            "walk", "walks", "walked", "walking",
            "win", "wins", "won", "winning",
            "offer", "offers", "offered", "offering",
            "remember", "remembers", "remembered", "remembering",
            "love", "loves", "loved", "loving",
            "consider", "considers", "considered", "considering",
            "appear", "appears", "appeared", "appearing",
            "buy", "buys", "bought", "buying",
            "wait", "waits", "waited", "waiting",
            "serve", "serves", "served", "serving",
            "die", "dies", "died", "dying",
            "send", "sends", "sent", "sending",
            "expect", "expects", "expected", "expecting",
            "build", "builds", "built", "building",
            "stay", "stays", "stayed", "staying",
            "fall", "falls", "fell", "fallen", "falling",
            "cut", "cuts", "cutting",
            "reach", "reaches", "reached", "reaching",
            "kill", "kills", "killed", "killing",
            "remain", "remains", "remained", "remaining",
            
            // Предлоги и союзы
            "в", "во", "на", "с", "со", "к", "ко", "у", "о", "об", "обо", "от", "ото", "до", "для", "за", "по", "из", "без",
            "под", "подо", "над", "надо", "перед", "между", "среди", "через", "при", "про", "ради", "вокруг", "около",
            "вдоль", "возле", "кроме", "сквозь", "после", "внутри", "вне", "вблизи",
            "и", "а", "но", "или", "да", "ни", "не", "ни...ни", "либо", "то...то",
            "если", "чтобы", "что", "когда", "как", "потому", "поэтому", "так", "тоже", "также", "тогда",
            "хотя", "несмотря", "пока", "едва", "лишь", "только", "даже", "ведь", "же",
            
            "in", "on", "at", "to", "for", "of", "with", "from", "by", "about", "as", "into", "like", "through",
            "after", "over", "between", "out", "against", "during", "without", "before", "under", "around",
            "among", "beyond", "near", "within", "above", "below", "across", "behind", "beside",
            "and", "or", "but", "if", "so", "than", "because", "while", "where", "though", "although", "since", "until",
            "unless", "whether", "nor", "yet", "either", "neither", "both",
            
            // Числительные и количество
            "ноль", "один", "одна", "одно", "два", "две", "три", "четыре", "пять", "шесть", "семь", "восемь", "девять", "десять",
            "одиннадцать", "двенадцать", "тринадцать", "четырнадцать", "пятнадцать",
            "двадцать", "тридцать", "сорок", "пятьдесят", "сто", "тысяча", "миллион",
            "первый", "второй", "третий", "четвёртый", "пятый", "последний",
            "много", "мало", "немного", "несколько", "сколько-нибудь", "немало",
            
            "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
            "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen",
            "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
            "hundred", "thousand", "million", "billion",
            "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth",
            "last", "next", "previous",
            "many", "much", "some", "few", "several", "all", "each", "every", "any", "no", "none",
            "more", "most", "less", "least", "enough",
            
            // Прилагательные (расширенные)
            "хороший", "плохой", "большой", "маленький", "новый", "старый", "молодой", "важный", "нужный",
            "правильный", "неправильный", "красивый", "простой", "сложный", "лёгкий", "тяжёлый", "трудный",
            "длинный", "короткий", "высокий", "низкий", "широкий", "узкий", "глубокий", "мелкий",
            "быстрый", "медленный", "ранний", "поздний", "близкий", "далёкий", "дальний",
            "полный", "пустой", "целый", "главный", "общий", "особый", "единственный", "другой",
            "различный", "одинаковый", "похожий", "разный", "отдельный", "каждый",
            "белый", "чёрный", "красный", "синий", "зелёный", "жёлтый", "серый", "голубой", "коричневый",
            "горячий", "холодный", "тёплый", "прохладный", "мокрый", "сухой", "чистый", "грязный",
            "живой", "мёртвый", "здоровый", "больной", "сильный", "слабый", "умный", "глупый",
            "добрый", "злой", "весёлый", "грустный", "счастливый", "несчастный",
            "богатый", "бедный", "дорогой", "дешёвый", "свободный", "занятый",
            "возможный", "невозможный", "необходимый", "обязательный",
            "русский", "английский", "немецкий", "французский", "китайский", "японский",
            
            "good", "bad", "great", "small", "big", "large", "new", "old", "young", "important",
            "high", "low", "long", "short", "tall", "wide", "narrow", "deep", "shallow",
            "fast", "quick", "slow", "early", "late", "near", "far", "close", "distant",
            "full", "empty", "whole", "main", "general", "common", "special", "particular", "certain",
            "different", "same", "similar", "various", "single", "separate", "individual",
            "right", "wrong", "correct", "incorrect", "true", "false",
            "easy", "hard", "difficult", "simple", "complex", "complicated",
            "beautiful", "pretty", "ugly", "nice", "fine",
            "white", "black", "red", "blue", "green", "yellow", "gray", "brown", "orange", "purple", "pink",
            "hot", "cold", "warm", "cool", "wet", "dry", "clean", "dirty",
            "alive", "dead", "living", "healthy", "sick", "strong", "weak", "smart", "stupid",
            "kind", "mean", "happy", "sad", "glad", "sorry",
            "rich", "poor", "expensive", "cheap", "free", "busy",
            "possible", "impossible", "necessary", "important", "essential",
            "ready", "sure", "certain", "clear", "obvious",
            "public", "private", "personal", "social", "political", "economic",
            "natural", "human", "real", "physical", "mental", "environmental",
            
            // Существительные (расширенные)
            "человек", "люди", "народ", "общество", "мужчина", "женщина", "ребёнок", "дети", "родитель",
            "мать", "отец", "сын", "дочь", "брат", "сестра", "семья", "друг", "враг", "товарищ",
            "время", "год", "месяц", "неделя", "день", "час", "минута", "секунда", "момент", "период",
            "жизнь", "смерть", "рождение", "детство", "юность", "молодость", "старость",
            "работа", "труд", "дело", "занятие", "деятельность", "профессия", "карьера", "бизнес",
            "место", "пространство", "область", "район", "зона", "территория", "площадь",
            "мир", "земля", "планета", "природа", "среда", "воздух", "небо", "облако", "солнце", "луна", "звезда",
            "страна", "государство", "нация", "столица", "город", "деревня", "село", "улица", "дорога", "путь",
            "дом", "здание", "квартира", "комната", "стена", "пол", "потолок", "окно", "дверь", "крыша",
            "стол", "стул", "кровать", "шкаф", "мебель", "вещь", "предмет", "объект",
            "тело", "часть", "рука", "нога", "голова", "лицо", "глаз", "ухо", "нос", "рот", "зуб",
            "сердце", "мозг", "кровь", "кожа", "волосы", "палец", "спина", "живот", "грудь", "плечо",
            "вода", "огонь", "земля", "камень", "металл", "дерево", "трава", "цветок", "растение", "лес",
            "еда", "пища", "хлеб", "мясо", "рыба", "молоко", "сыр", "масло", "соль", "сахар",
            "фрукт", "овощ", "яблоко", "картофель", "помидор", "огурец",
            "напиток", "вино", "пиво", "чай", "кофе", "сок",
            "деньги", "цена", "стоимость", "зарплата", "доход", "расход", "налог", "бюджет",
            "вопрос", "ответ", "проблема", "решение", "задача", "цель", "результат", "успех", "неудача",
            "слово", "фраза", "предложение", "текст", "язык", "речь", "голос", "звук", "тишина",
            "книга", "страница", "буква", "знак", "письмо", "документ", "бумага",
            "история", "событие", "факт", "причина", "следствие", "условие", "случай",
            "система", "структура", "элемент", "компонент", "часть", "целое", "единица",
            "наука", "знание", "информация", "данные", "теория", "практика", "опыт", "исследование",
            "образование", "школа", "университет", "учитель", "ученик", "студент", "урок", "экзамен",
            "искусство", "культура", "музыка", "песня", "картина", "фильм", "театр", "игра",
            "закон", "право", "правило", "норма", "порядок", "власть", "правительство", "президент",
            "война", "мир", "армия", "солдат", "оружие", "битва", "победа", "поражение",
            "здоровье", "болезнь", "лечение", "лекарство", "врач", "больница", "медицина",
            "транспорт", "машина", "автомобиль", "поезд", "самолёт", "корабль", "велосипед",
            "связь", "телефон", "письмо", "почта", "сообщение", "новость", "газета",
            
            "people", "person", "man", "woman", "child", "children", "parent", "family", "friend",
            "father", "mother", "son", "daughter", "brother", "sister", "husband", "wife",
            "time", "year", "month", "week", "day", "hour", "minute", "second", "moment", "period",
            "life", "death", "birth", "age",
            "work", "job", "business", "company", "office", "worker", "employee", "boss", "manager",
            "world", "earth", "nature", "environment", "air", "sky", "sun", "moon", "star", "planet",
            "country", "nation", "state", "city", "town", "village", "street", "road", "way",
            "home", "house", "building", "room", "door", "window", "wall", "floor", "ceiling",
            "table", "chair", "bed", "furniture", "thing", "object", "item", "stuff",
            "body", "part", "hand", "arm", "leg", "foot", "head", "face", "eye", "ear", "nose", "mouth",
            "heart", "brain", "blood", "skin", "hair", "finger", "back", "stomach", "chest", "shoulder",
            "water", "fire", "earth", "stone", "metal", "wood", "tree", "grass", "flower", "plant", "forest",
            "food", "bread", "meat", "fish", "milk", "cheese", "butter", "salt", "sugar",
            "fruit", "vegetable", "apple", "potato", "tomato",
            "drink", "wine", "beer", "tea", "coffee", "juice",
            "money", "price", "cost", "value", "pay", "salary", "income", "tax", "budget",
            "question", "answer", "problem", "solution", "task", "goal", "result", "success", "failure",
            "word", "sentence", "text", "language", "speech", "voice", "sound", "noise", "silence",
            "book", "page", "letter", "sign", "paper", "document", "report",
            "history", "story", "event", "fact", "reason", "cause", "effect", "condition", "case",
            "system", "structure", "element", "component", "unit",
            "science", "knowledge", "information", "data", "theory", "practice", "experience", "research",
            "education", "school", "university", "college", "teacher", "student", "lesson", "class", "course",
            "art", "culture", "music", "song", "picture", "painting", "film", "movie", "theater", "game",
            "law", "rule", "order", "power", "government", "president", "minister",
            "war", "peace", "army", "soldier", "weapon", "battle", "victory", "defeat",
            "health", "disease", "illness", "treatment", "medicine", "doctor", "hospital",
            "transport", "car", "vehicle", "train", "plane", "ship", "boat", "bicycle", "bike",
            "communication", "phone", "telephone", "mail", "email", "message", "news", "newspaper",
            
            // IT, программирование, AI
            "программа", "программирование", "программист", "разработчик", "разработка",
            "код", "кодирование", "исходный", "компиляция", "компилятор",
            "файл", "папка", "директория", "путь", "расширение",
            "данные", "база", "массив", "список", "словарь", "множество",
            "система", "операционный", "windows", "linux", "macos", "unix",
            "компьютер", "процессор", "память", "диск", "монитор", "клавиатура", "мышь",
            "интернет", "сеть", "сервер", "клиент", "протокол", "ip", "tcp", "http", "https",
            "сайт", "веб", "страница", "браузер", "ссылка", "url",
            "приложение", "софт", "программное", "обеспечение",
            "функция", "метод", "процедура", "подпрограмма",
            "переменная", "константа", "параметр", "аргумент",
            "класс", "объект", "экземпляр", "наследование", "полиморфизм", "инкапсуляция",
            "алгоритм", "сложность", "оптимизация", "рефакторинг",
            "язык", "python", "rust", "javascript", "js", "java", "cpp", "c", "go", "ruby", "php",
            "фреймворк", "библиотека", "модуль", "пакет", "зависимость",
            "git", "github", "репозиторий", "коммит", "ветка", "мерж", "пулл-реквест",
            "тест", "тестирование", "отладка", "дебаг", "баг", "ошибка", "исключение",
            "документация", "комментарий", "readme",
            
            "program", "programming", "programmer", "developer", "development",
            "code", "coding", "source", "compilation", "compiler", "interpreter",
            "file", "folder", "directory", "path", "extension",
            "data", "database", "array", "list", "dict", "dictionary", "set",
            "system", "operating", "windows", "linux", "macos", "unix",
            "computer", "processor", "cpu", "memory", "ram", "disk", "monitor", "keyboard", "mouse",
            "internet", "network", "server", "client", "protocol", "ip", "tcp", "http", "https", "api",
            "website", "web", "page", "browser", "link", "url",
            "application", "app", "software",
            "function", "method", "procedure", "subroutine",
            "variable", "constant", "parameter", "argument", "arg",
            "class", "object", "instance", "inheritance", "polymorphism", "encapsulation",
            "algorithm", "complexity", "optimization", "refactoring",
            "language", "python", "rust", "javascript", "js", "java", "cpp", "c", "go", "ruby", "php", "sql",
            "framework", "library", "module", "package", "dependency",
            "git", "github", "repository", "repo", "commit", "branch", "merge", "pull", "request", "pr",
            "test", "testing", "debug", "debugging", "bug", "error", "exception",
            "documentation", "comment", "readme", "docs",
            
            // AI и машинное обучение (расширенные)
            "модель", "обучение", "тренировка", "дообучение", "переобучение",
            "нейронный", "нейрон", "сеть", "слой", "вес", "смещение", "активация",
            "данные", "датасет", "выборка", "обучающий", "тестовый", "валидационный",
            "алгоритм", "градиент", "спуск", "оптимизатор", "функция", "потерь",
            "искусственный", "интеллект", "машинный", "глубокий", "глубинный",
            "признак", "метка", "класс", "регрессия", "классификация", "кластеризация",
            "точность", "precision", "recall", "f1", "метрика", "ошибка",
            "эпоха", "батч", "шаг", "итерация", "learning_rate", "скорость",
            "embedding", "эмбеддинг", "представление", "вектор", "матрица", "тензор",
            "трансформер", "attention", "внимание", "self-attention",
            "llm", "gpt", "bert", "transformer", "lstm", "rnn", "cnn",
            "токен", "токенизация", "словарь", "vocab", "vocabulary",
            "inference", "инференс", "предсказание", "генерация",
            "fine-tuning", "дообучение", "transfer", "learning",
            "supervised", "unsupervised", "reinforcement", "контроль", "подкрепление",
            
            "model", "training", "train", "learning", "learn", "overfitting", "underfitting",
            "neural", "neuron", "network", "layer", "weight", "bias", "activation",
            "data", "dataset", "sample", "training", "test", "validation",
            "algorithm", "gradient", "descent", "optimizer", "loss", "function",
            "artificial", "intelligence", "machine", "deep", "learning",
            "feature", "label", "class", "regression", "classification", "clustering",
            "accuracy", "precision", "recall", "f1", "metric", "error",
            "epoch", "batch", "step", "iteration", "learning_rate", "rate",
            "embedding", "representation", "vector", "matrix", "tensor",
            "transformer", "attention", "self-attention", "multi-head",
            "llm", "gpt", "bert", "lstm", "rnn", "cnn", "gan",
            "token", "tokenization", "vocabulary", "vocab",
            "inference", "prediction", "generation", "generate",
            "fine-tuning", "transfer", "learning",
            "supervised", "unsupervised", "reinforcement",
            "fp64", "fp32", "fp16", "float", "precision", "double", "half",
            
            // Наречия
            "хорошо", "плохо", "очень", "совсем", "довольно", "слишком", "почти", "едва", "чуть",
            "быстро", "медленно", "долго", "скоро", "рано", "поздно", "сейчас", "теперь", "тогда",
            "здесь", "тут", "там", "туда", "сюда", "везде", "нигде",
            "всегда", "никогда", "иногда", "часто", "редко", "обычно", "обыч", "снова", "опять", "ещё",
            "уже", "ещё", "только", "лишь", "даже", "именно", "просто", "прямо",
            "вместе", "отдельно", "вдруг", "сразу", "немедленно", "постепенно",
            
            "well", "good", "bad", "very", "too", "quite", "almost", "nearly", "just", "only",
            "fast", "quickly", "slow", "slowly", "long", "soon", "early", "late", "now", "then",
            "here", "there", "everywhere", "nowhere", "somewhere", "anywhere",
            "always", "never", "sometimes", "often", "rarely", "usually", "again", "still", "yet",
            "already", "just", "only", "even", "exactly", "simply", "directly",
            "together", "apart", "suddenly", "immediately", "gradually",
            
            // Служебные токены и специальные символы
            "<PAD>", "<START>", "<END>", "<UNK>", "<MASK>", "<SEP>", "<CLS>",
            "!", "?", ".", ",", ";", ":", "-", "–", "—",
            "(", ")", "[", "]", "{", "}", "\"", "'", "`",
            "/", "\\", "|", "@", "#", "$", "%", "^", "&", "*", "+", "=", "<", ">", "~",
        ];
        
        for (idx, word) in base_words.iter().enumerate() {
            self.vocab.insert(word.to_string(), idx);
            self.reverse_vocab.insert(idx, word.to_string());
        }
    }
    
    fn init_layers(&mut self) {
        let mut rng = rand::thread_rng();
        let vocab_size = self.vocab.len();
        
        // Embedding layer
        let embedding_layer = Layer {
            weights: (0..vocab_size)
                .map(|_| (0..self.embedding_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect())
                .collect(),
            biases: vec![0.0; self.embedding_dim],
            activation: ActivationType::ReLU,
        };
        
        // Hidden layer 1
        let hidden1 = Layer {
            weights: (0..self.embedding_dim * self.context_length)
                .map(|_| (0..self.hidden_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect())
                .collect(),
            biases: vec![0.0; self.hidden_dim],
            activation: ActivationType::Tanh,
        };
        
        // Hidden layer 2
        let hidden2 = Layer {
            weights: (0..self.hidden_dim)
                .map(|_| (0..self.hidden_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect())
                .collect(),
            biases: vec![0.0; self.hidden_dim],
            activation: ActivationType::Tanh,
        };
        
        // Output layer
        let output_layer = Layer {
            weights: (0..self.hidden_dim)
                .map(|_| (0..vocab_size)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect())
                .collect(),
            biases: vec![0.0; vocab_size],
            activation: ActivationType::Softmax,
        };
        
        self.layers.push(embedding_layer);
        self.layers.push(hidden1);
        self.layers.push(hidden2);
        self.layers.push(output_layer);
    }
    
    /// Прямое распространение
    pub fn forward(&self, input_tokens: &[usize]) -> Vec<f64> {
        let mut activations = Vec::new();
        
        // Embedding
        for &token in input_tokens.iter().take(self.context_length) {
            if token < self.layers[0].weights.len() {
                activations.extend_from_slice(&self.layers[0].weights[token]);
            } else {
                activations.extend(vec![0.0; self.embedding_dim]);
            }
        }
        
        // Дополняем до нужной длины
        while activations.len() < self.embedding_dim * self.context_length {
            activations.push(0.0);
        }
        
        // Проход через скрытые слои
        for layer in self.layers.iter().skip(1) {
            activations = self.apply_layer(&activations, layer);
        }
        
        activations
    }
    
    fn apply_layer(&self, input: &[f64], layer: &Layer) -> Vec<f64> {
        let output_size = layer.biases.len();
        let input_size = if layer.weights.is_empty() { 0 } else { layer.weights[0].len() };
        
        let mut output = vec![0.0; output_size];
        
        for i in 0..output_size {
            let mut sum = layer.biases[i];
            for j in 0..input.len().min(layer.weights.len()) {
                if i < layer.weights[j].len() {
                    sum += input[j] * layer.weights[j][i];
                }
            }
            output[i] = sum;
        }
        
        // Применение функции активации
        match layer.activation {
            ActivationType::ReLU => output.iter().map(|&x| x.max(0.0)).collect(),
            ActivationType::Tanh => output.iter().map(|&x| x.tanh()).collect(),
            ActivationType::Sigmoid => output.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect(),
            ActivationType::Softmax => {
                let max_val = output.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let exp_vals: Vec<f64> = output.iter().map(|&x| (x - max_val).exp()).collect();
                let sum: f64 = exp_vals.iter().sum();
                exp_vals.iter().map(|&x| x / sum).collect()
            }
        }
    }
    
    /// Генерация ответа
    pub fn generate(&self, input_text: &str, max_length: usize) -> String {
        let tokens = self.tokenize(input_text);
        let mut generated_tokens = tokens.clone();
        
        for _ in 0..max_length {
            let context: Vec<usize> = generated_tokens
                .iter()
                .rev()
                .take(self.context_length)
                .rev()
                .cloned()
                .collect();
            
            let probs = self.forward(&context);
            let next_token = self.sample_token(&probs);
            
            // Проверка на конец генерации
            if let Some(token_str) = self.reverse_vocab.get(&next_token) {
                if token_str == "<END>" {
                    break;
                }
            }
            
            generated_tokens.push(next_token);
        }
        
        self.decode(&generated_tokens[tokens.len()..])
    }
    
    /// Обучение на данных
    pub fn train(&mut self, texts: &[String], epochs: usize, progress_callback: impl Fn(usize, usize, f64)) {
        for epoch in 0..epochs {
            let mut total_loss = 0.0;
            let mut num_samples = 0;
            
            for text in texts {
                let tokens = self.tokenize(text);
                
                // Создаем обучающие пары (контекст -> следующее слово)
                for i in 0..(tokens.len().saturating_sub(1)) {
                    let context_end = (i + 1).min(tokens.len());
                    let context_start = context_end.saturating_sub(self.context_length);
                    let context = &tokens[context_start..context_end];
                    let target = tokens[context_end.min(tokens.len() - 1)];
                    
                    // Forward pass
                    let output = self.forward(context);
                    
                    // Вычисление loss
                    let loss = self.compute_loss(&output, target);
                    total_loss += loss;
                    num_samples += 1;
                    
                    // Backward pass (упрощенный градиентный спуск)
                    self.update_weights(context, target, &output);
                }
            }
            
            let avg_loss = if num_samples > 0 { total_loss / num_samples as f64 } else { 0.0 };
            progress_callback(epoch + 1, epochs, avg_loss);
        }
    }
    
    fn compute_loss(&self, output: &[f64], target: usize) -> f64 {
        if target >= output.len() {
            return 1.0;
        }
        // Cross-entropy loss
        -output[target].ln()
    }
    
    fn update_weights(&mut self, context: &[usize], target: usize, output: &[f64]) {
        // Упрощенный градиентный спуск
        // В реальной реализации здесь был бы полный backpropagation
        let lr = self.learning_rate;
        
        if target >= output.len() || self.layers.is_empty() {
            return;
        }
        
        // Обновление весов выходного слоя
        let output_layer_idx = self.layers.len() - 1;
        if output_layer_idx < self.layers.len() {
            let error = output[target] - 1.0; // gradient
            
            // Простое обновление bias
            if target < self.layers[output_layer_idx].biases.len() {
                self.layers[output_layer_idx].biases[target] -= lr * error;
            }
        }
    }
    
    fn sample_token(&self, probs: &[f64]) -> usize {
        let mut rng = rand::thread_rng();
        let random_val: f64 = rng.gen();
        let mut cumsum = 0.0;
        
        for (idx, &prob) in probs.iter().enumerate() {
            cumsum += prob;
            if random_val < cumsum {
                return idx;
            }
        }
        
        probs.len().saturating_sub(1)
    }
    
    /// Токенизация текста
    pub fn tokenize(&self, text: &str) -> Vec<usize> {
        text.split_whitespace()
            .map(|word| {
                let word_lower = word.to_lowercase();
                *self.vocab.get(&word_lower).unwrap_or(&self.get_unk_token())
            })
            .collect()
    }
    
    fn get_unk_token(&self) -> usize {
        *self.vocab.get("<UNK>").unwrap_or(&0)
    }
    
    /// Декодирование токенов в текст
    pub fn decode(&self, tokens: &[usize]) -> String {
        tokens
            .iter()
            .filter_map(|&token| self.reverse_vocab.get(&token))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Добавление нового слова в словарь
    pub fn add_to_vocab(&mut self, word: String) {
        if !self.vocab.contains_key(&word) {
            let idx = self.vocab.len();
            self.vocab.insert(word.clone(), idx);
            self.reverse_vocab.insert(idx, word);
            
            // Расширяем embedding layer
            let mut rng = rand::thread_rng();
            if !self.layers.is_empty() {
                let new_embedding: Vec<f64> = (0..self.embedding_dim)
                    .map(|_| rng.gen_range(-0.1..0.1))
                    .collect();
                self.layers[0].weights.push(new_embedding);
            }
        }
    }
    
    /// Сохранение модели
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string(self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }
    
    /// Загрузка модели
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(path)?;
        let model = serde_json::from_str(&data)?;
        Ok(model)
    }
    
    /// Получение информации о модели
    pub fn info(&self) -> String {
        format!(
            "Модель AI (fp64)\n\
             Словарь: {} слов\n\
             Embedding dimension: {}\n\
             Hidden dimension: {}\n\
             Context length: {}\n\
             Слои: {}\n\
             Learning rate: {}",
            self.vocab.len(),
            self.embedding_dim,
            self.hidden_dim,
            self.context_length,
            self.layers.len(),
            self.learning_rate
        )
    }
}

impl Default for AIModel {
    fn default() -> Self {
        Self::new(128, 256, 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_creation() {
        let model = AIModel::new(64, 128, 4);
        assert_eq!(model.embedding_dim, 64);
        assert_eq!(model.hidden_dim, 128);
        assert_eq!(model.context_length, 4);
    }
    
    #[test]
    fn test_tokenization() {
        let model = AIModel::default();
        let tokens = model.tokenize("привет как дела");
        assert!(!tokens.is_empty());
    }
    
    #[test]
    fn test_generation() {
        let model = AIModel::default();
        let response = model.generate("привет", 5);
        assert!(!response.is_empty());
    }
}
