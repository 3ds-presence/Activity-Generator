-- Script for Hatsune Miku Project Mirai DX (Europe)

-- Song/level mapping (key = decimal, result of hex_to_num(get("006098E0")))
local songs = {
    [28]  = "Finder (DSLR remix - re:edit)",
    [54]  = "Sweet Magic",
    [16]  = "Deep Sea Girl",
    [4]   = "Animal Fortune-telling",
    [67]  = "Terekakushi Shishunki",
    [58]  = "The World is Mine",
    [72]  = "Amatsu Kitsune",
    [65]  = "Ageage Again",
    [22]  = "Clover♣Club",
    [63]  = "Yumeyume",
    [73]  = "Doremifa Rondo",
    [24]  = "reverse rainbow",
    [57]  = "KONEKO NO PAYAPAYA",
    [32]  = "Hello/How are you?",
    [43]  = "Kokoro",
    [11]  = "PIANO*GIRL",
    [31]  = "Happy Synthesizer",
    [21]  = "SING&SMILE",
    [53]  = "1/6 -out of the gravity-",
    [25]  = "Mousou Sketch",
    [42]  = "1925",
    [13]  = "Matryoshka",
    [59]  = "Cendrillon",
    [64]  = "Adolescence",
    [26]  = "on the rocks",
    [27]  = "No Logic",
    [74]  = "Hello, Planet.",
    [61]  = "Romeo and Cinderella",
    [14]  = "LOL -lots of laugh-",
    [62]  = "Sebonzakura",
    [2]   = "Aku no Musume",
    [3]   = "Aku no Meshitsukai",
    [66]  = "Snowman",
    [71]  = "Invisible",
    [52]  = "Gaikotsu Gakudan to Riria",
    [75]  = "ARIFURETA SEKAI SEIFUKU",
    [41]  = "Electric Love",
    [12]  = "Melancholic",
    [56]  = "1 2 Fanclub",
    [60]  = "Electric Angel",
    [45]  = "Interviewer",
    [23]  = "Tricolore Airline",
    [55]  = "Piano✕Forte✕Scandal",
    [15]  = "Kimi no Taion",
    [44]  = "glow",
}

-- Difficulty mapping (key = decimal from hex_to_num on 004FE704)
local difficulties = {
    [0] = "Easy",
    [1] = "Normal",
    [2] = "Hard",
    [3] = "Super Hard",
    [5] = "Theater",
}

-- Game mode mapping (key = decimal from hex_to_num on 004EF580)
local modes = {
    [0] = "Tap Mode",
    [1] = "Button Mode",
}

function build(game_info, extra_info)
    -- Get current song/level: get() gets the hex string, hex_to_num() converts it
    local level_id = hex_to_num(get("006098E0"))
    local song = songs[level_id]
    if not song then
        fallback()
        return nil
    end

    -- Get current difficulty
    local diff_id = hex_to_num(get("004FE704"))
    local diff = difficulties[diff_id]
    if not diff then
        fallback()
        return nil
    end

    local details
    if diff == "Theater" then
        details = "Theater"
    else
        local mode_id = hex_to_num(get("004EF580"))
        local mode = modes[mode_id]
        if not mode then
            fallback()
            return nil
        end
        local diff = difficulties[diff_id]
        details = mode .. " | " .. diff
    end

    return {
        name = song .. " - Project Mirai DX",
        activity_type = 2,
        details = details,
    }
end