# Navigation Arrows & Custom Emojis Policy

## MANDATORY ARROW EMOJIS (STRICT REQUIREMENT)
Whenever rendering navigation arrows, pagination controls, or directional arrows anywhere in this bot (current features and ALL future features), you MUST use the following custom Telegram emoji IDs:

- **Left Arrow (`←`)**:
  - Custom Emoji ID: `5283240108328244731`
  - Constant: `LEFT_ARROW_CUSTOM_EMOJI_ID`
  - HTML format: `<tg-emoji emoji-id="5283240108328244731">←</tg-emoji>` (`LEFT_ARROW_TG_EMOJI`)
- **Right Arrow (`→`)**:
  - Custom Emoji ID: `5282981997973621943`
  - Constant: `RIGHT_ARROW_CUSTOM_EMOJI_ID`
  - HTML format: `<tg-emoji emoji-id="5282981997973621943">→</tg-emoji>` (`RIGHT_ARROW_TG_EMOJI`)

## Inline Keyboard Buttons
- For all pagination keyboards (`mypokemon`, `pokedexall`, and any future paginated views), Row 1 navigation buttons must use `icon_custom_emoji_id`:
  ```json
  {
      "text": " ",
      "callback_data": "<prefix>:<owner_id>:<prev_page>",
      "icon_custom_emoji_id": "5283240108328244731"
  }
  ```
  and
  ```json
  {
      "text": " ",
      "callback_data": "<prefix>:<owner_id>:<next_page>",
      "icon_custom_emoji_id": "5282981997973621943"
  }
  ```
- Because Teloxide's native `InlineKeyboardButton` does not expose `icon_custom_emoji_id`, custom keyboards must be sent via raw JSON payload through `sendMessage` and `editMessageText`.
- **NEVER** replace or revert these custom arrow emojis to standard Unicode emojis (`⬅️`, `➡️`).
