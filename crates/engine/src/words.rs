//! The five letter words the reading games are built from.

/// Every word is five letters, lower case, appears once, and is spelled with
/// a set of letters no other word here shares, so the letters of a word name
/// exactly one word in this list. No word reads the same backwards either, so
/// a word laid on a grid can be read in one direction only.
pub const WORDS: &[&str] = &[
    "about", "above", "abuse", "actor", "acute", "admit", "adopt", "adult", "after", "again",
    "agent", "agree", "ahead", "alarm", "album", "alert", "alike", "alive", "allow", "alone",
    "along", "amber", "among", "anger", "angle", "angry", "ankle", "apart", "apple", "apply",
    "apron", "arena", "argue", "arise", "armor", "array", "arrow", "aside", "asset", "audio",
    "audit", "avoid", "awake", "award", "aware", "bacon", "badge", "badly", "baker", "basic",
    "basis", "beach", "beard", "beast", "begin", "below", "bench", "birth", "bison", "black",
    "blade", "blame", "blank", "blast", "blaze", "blend", "bless", "blind", "block", "blood",
    "bloom", "board", "bonus", "boost", "booth", "bound", "brain", "brand", "brass", "brave",
    "breed", "brick", "brief", "bring", "brisk", "broke", "brown", "brush", "build", "built",
    "bunch", "burst", "cabin", "cable", "cadet", "camel", "canal", "candy", "cargo", "carry",
    "catch", "cause", "cedar", "chain", "chair", "chalk", "chaos", "charm", "chart", "chase",
    "cheap", "check", "cheek", "chess", "chest", "chief", "child", "chirp", "choir", "claim",
    "class", "clean", "clear", "clerk", "click", "cliff", "climb", "cloak", "clock", "close",
    "cloth", "cloud", "coach", "coast", "cocoa", "comet", "coral", "count", "court", "cover",
    "crack", "craft", "crash", "crazy", "cream", "crime", "crisp", "cross", "crowd", "crown",
    "crude", "curve", "cycle", "daily", "dairy", "dance", "death", "debut", "delay", "delta",
    "dense", "depth", "dodge", "doubt", "dozen", "draft", "drain", "drama", "drank", "dream",
    "dress", "drift", "drink", "drive", "drove", "early", "earth", "eight", "elder", "elect",
    "elite", "ember", "empty", "enemy", "enjoy", "enter", "entry", "equal", "error", "event",
    "every", "exact", "exist", "extra", "fable", "faith", "false", "fault", "favor", "fence",
    "fever", "field", "fifth", "fight", "final", "first", "flame", "flash", "fleet", "flesh",
    "float", "flood", "floor", "flour", "fluid", "focus", "force", "forge", "forth", "forty",
    "forum", "found", "frame", "fraud", "fresh", "front", "frost", "fruit", "fudge", "fully",
    "funny", "gauge", "ghost", "giant", "given", "glass", "glide", "globe", "glory", "grace",
    "grade", "grain", "grand", "grant", "grape", "graph", "grasp", "grass", "grave", "great",
    "green", "greet", "grief", "group", "grown", "guard", "guess", "guest", "guide", "happy",
    "harsh", "heavy", "hedge", "hello", "hobby", "honey", "honor", "horse", "hotel", "house",
    "human", "humor", "hurry", "ideal", "igloo", "image", "imply", "index", "inner", "input",
    "irony", "issue", "ivory", "jewel", "joint", "judge", "juice", "jumbo", "karma", "kneel",
    "knife", "knock", "known", "label", "labor", "large", "laser", "laugh", "learn", "lease",
    "least", "leave", "legal", "lemon", "light", "lilac", "limit", "linen", "liver", "lobby",
    "local", "logic", "loose", "lower", "lucky", "lunar", "lunch", "magic", "major", "maker",
    "maple", "match", "maybe", "mayor", "meant", "medal", "media", "mercy", "merit", "metal",
    "meter", "might", "minor", "minus", "mirth", "mixed", "model", "moist", "money", "month",
    "moral", "motor", "mount", "mouse", "mouth", "movie", "music", "naked", "nerve", "newly",
    "night", "noble", "noise", "north", "novel", "nurse", "nylon", "ocean", "offer", "often",
    "onion", "opera", "order", "other", "otter", "outer", "owner", "paint", "panel", "paper",
    "party", "pasta", "patch", "pause", "peace", "pearl", "phase", "phone", "photo", "piano",
    "piece", "pilot", "pitch", "pixel", "place", "plain", "plant", "plate", "point", "polar",
    "pound", "power", "prawn", "press", "price", "pride", "prime", "print", "prize", "probe",
    "proof", "proud", "prove", "pulse", "punch", "pupil", "purse", "queen", "query", "quest",
    "queue", "quick", "quiet", "quilt", "quota", "radio", "rally", "rapid", "ratio", "raven",
    "reach", "ready", "realm", "rebel", "relax", "renew", "reply", "rhyme", "rider", "ridge",
    "rifle", "right", "rigid", "rival", "river", "roast", "robot", "rocky", "rough", "round",
    "royal", "rugby", "rural", "saint", "salad", "scale", "scene", "scope", "score", "scout",
    "screw", "sedan", "sense", "serve", "seven", "shade", "shaft", "shake", "shall", "share",
    "sharp", "sheep", "sheet", "shell", "shift", "shine", "shirt", "shock", "shoot", "short",
    "shown", "sight", "silly", "since", "siren", "skill", "sleep", "slice", "slide", "slope",
    "small", "smart", "smell", "smile", "smoke", "snake", "solar", "solid", "solve", "sorry",
    "sound", "south", "space", "spare", "speak", "speed", "spell", "spend", "spent", "spice",
    "spike", "spine", "split", "spoke", "sport", "spray", "squad", "staff", "stage", "stair",
    "stake", "stamp", "stand", "stare", "start", "state", "steam", "steel", "steep", "steer",
    "stern", "stick", "still", "stock", "stone", "stood", "store", "storm", "story", "strap",
    "straw", "strip", "stuck", "study", "stuff", "style", "sugar", "suite", "sunny", "swear",
    "sweat", "sweep", "sweet", "swept", "swift", "swing", "sword", "table", "taken", "teach",
    "teeth", "tempo", "tenth", "thank", "theft", "theme", "there", "thick", "thief", "think",
    "third", "those", "throw", "thumb", "tiger", "tight", "tired", "title", "toast", "today",
    "token", "tooth", "topic", "total", "touch", "tough", "tower", "toxic", "trace", "track",
    "trade", "trail", "train", "trait", "trash", "treat", "trend", "tribe", "trick", "tulip",
    "twice", "twist", "udder", "ulcer", "ultra", "uncle", "under", "union", "unite", "unity",
    "until", "upper", "upset", "urban", "usage", "usual", "valid", "value", "vapor", "vault",
    "video", "villa", "vinyl", "virus", "visit", "vital", "vivid", "vocal", "voice", "voter",
    "wafer", "wagon", "waist", "watch", "water", "wheat", "wheel", "where", "which", "while",
    "white", "whole", "whose", "widen", "width", "witch", "woman", "world", "worry", "worse",
    "worst", "would", "wound", "wrist", "write", "yacht", "yield", "young", "youth", "zebra",
];

/// The letters of a word sorted, which every anagram of it shares.
pub fn signature(word: &str) -> [u8; 5] {
    let mut letters = [0u8; 5];
    for (slot, letter) in word.bytes().take(5).enumerate() {
        letters[slot] = letter;
    }
    letters.sort_unstable();
    letters
}

#[cfg(test)]
#[path = "tests/words.rs"]
mod tests;
