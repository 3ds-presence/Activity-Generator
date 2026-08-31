# Presence-3DS Activity Generator
This project is part of the [Presence-3DS](https://github.com/3ds-presence/) project.

Generate the activity for Discord Rich Presence based on the current game running on the 3DS. The 3DS connects to the backend API to transfer the current game to Activity-Generator, the Activity-Generator then generates the activity and sends it to the backend, which then sends it to Discord.

## Basic RPC
By default, the Activity-Generator will generate a basic activity with the game name, publisher and the game logo. The game name and publisher are retrieved from the 3DS, and the game logo is retrieved from the frontend repo.

Also, if the user enables the functionality, the Activity-Generator will generate a small image with the Mii of the user, and will use it as the small image of the activity.

## Advanced RPC
The Activity-Generator can also generate an advanced activity with Lua Scripts. The Lua scripts are stored in the `scripts` folder, and can be used to generate custom activities based on the current game running on the 3DS. The 3DS must send some extra data to the backend API.

### Currently supported games
Following games have custom scripts to generate advanced activities:
- Hatsune Miku - Project Mirai DX (European version - 0004000000148C00)

### Scripts guide
The script process is divided into 2 parts: the 3DS side and the backend side. The 3DS side is responsible for sending the extra data to the backend, and the backend side is responsible for generating the activity based on that extra data. Both sides are required 

You can see some examples of scripts in the `scripts` folder.

#### Create 3DS script
The 3DS side is really simple, you just need to create a file named `<titleid>/code.txt` in the `scripts` folder. The file must contain the RAM addresses of the values you want to send to the backend, followed by the length (b = byte (8 bits), h = halfword (16 bits), w = word (32 bits)), separated by commas

Only the stack is supported to create add-ons, since the heap is too random (maybe supported in the future by using pointers ???).

##### Example
```
006098E0b,004FE704b,004EF580b
``` 
will send the values at the RAM addresses `0x006098E0`, `0x004FE704` and `0x004EF580` as bytes to the backend.

##### Find RAM addresses
This is the hardest part: you need to search in the RAM of the game to find the values you want to send.
I recommend using [CTR Plugin Framework](https://github.com/PabloMK7/CTRPluginFramework-BlankTemplate) on the 3DS. With this tool, you can use the `search` menu to search for a value, change the value in the game, and then search again to find the right one. 

#### Create server script
Scripts are named `<titleid>/script.lua` and are stored in the `scripts` folder. Each script must have a `build(game_info, extra_info)` function that returns a table representing the Discord Activity.

- `game_info` is a Lua table:
  - `title_id` (string) — e.g. `"0004000000148900"`
  - `name` (string) — game name
  - `publisher` (string) — publisher

- `extra_info` — Lua table (string keys/values) parsed from the query string sent by the 3DS:
  - Example: `006098E0=47&004FE704=01&004EF580=00`
  - Becomes `{ ["006098E0"] = "47", ["004FE704"] = "01", ["004EF580"] = "00" }` (all values are strings)

##### Available Helper Functions
- `require(key)`
  - Gets a string value from `extra_info`.
  - If the key is missing, the script is aborted and the default activity is used.
- `optional(key)` 
  - Gets a string value or returns `nil` if the key is missing.
- `hex_to_num(key)` 
  - Gets a hex string value from `extra_info` and converts it to a number. Falls back if the key is missing or invalid hex.
- `fallback()`
  - Explicitly request the default activity.

##### Returned Activity Table
The returned table must have the following structure:
```lua
{
    name = "Game Name",             -- REQUIRED
    activity_type = 0,              -- optional (0=Playing, 2=Listening, 3=Watching, 5=Competing)
    details = "Nintendo 3DS",       -- optional (1st line below name)
    state = "Via 3ds-presence.top", -- optional (2nd line below name)
    assets = {                      -- optional
        large_image = "url",
        large_text = "Hover text",
        small_image = "url",
        small_text = "Hover text"
    },
    timestamps = {                  -- optional
        start = 1234567890000,
        end = 1234567899999     
    }
}